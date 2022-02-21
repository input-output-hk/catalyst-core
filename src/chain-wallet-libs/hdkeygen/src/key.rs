use chain_path_derivation::{
    Derivation, DerivationPath, DerivationRange, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::{DerivationScheme, Signature, XPrv, XPub};
use std::fmt::{self, Debug, Display};

/// convenient wrapper around the `Key`.
///
pub struct Key<K, P> {
    key: K,
    path: DerivationPath<P>,
    derivation_scheme: DerivationScheme,
}

pub struct KeyRange<'a, K, DR, P, Q> {
    key: &'a Key<K, P>,
    range: DR,
    _marker: std::marker::PhantomData<Q>,
}

impl<'a, K, DR, P, Q> KeyRange<'a, K, DR, P, Q> {
    pub(crate) fn new(key: &'a Key<K, P>, range: DR) -> Self {
        Self {
            key,
            range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<K, P> Key<K, P> {
    /// create a `Key` with the given component
    ///
    /// does not guarantee that the derivation path is actually the one
    /// that lead to this key derivation.
    #[inline]
    pub fn new_unchecked(
        key: K,
        path: DerivationPath<P>,
        derivation_scheme: DerivationScheme,
    ) -> Self {
        Self {
            key,
            path,
            derivation_scheme,
        }
    }

    /// get the derivation path that lead to this key
    pub fn path(&self) -> &DerivationPath<P> {
        &self.path
    }

    pub fn coerce_unchecked<Q>(self) -> Key<K, Q> {
        Key {
            path: self.path.coerce_unchecked(),
            key: self.key,
            derivation_scheme: self.derivation_scheme,
        }
    }
}

impl<P> Key<XPrv, P> {
    /// retrieve the associated public key of the given private key
    ///
    #[inline]
    pub fn public(&self) -> Key<XPub, P> {
        Key {
            key: self.key.public(),
            path: self.path.clone(),
            derivation_scheme: self.derivation_scheme,
        }
    }

    /// create a signature for the given message and associate the given type `T`
    /// to the signature type.
    ///
    #[inline]
    pub fn sign<T, B>(&self, message: B) -> Signature<T>
    where
        B: AsRef<[u8]>,
    {
        self.key.sign(message.as_ref())
    }

    /// verify the signature with the private key for the given message
    #[inline]
    pub fn verify<T, B>(&self, message: B, signature: &Signature<T>) -> bool
    where
        B: AsRef<[u8]>,
    {
        self.key.verify(message.as_ref(), signature)
    }

    /// get key's chain code
    #[inline]
    pub fn chain_code(&self) -> [u8; 32] {
        *self.key.chain_code()
    }

    /// derive the private key against the given derivation index and scheme
    ///
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub(crate) fn derive_unchecked<Q>(&self, derivation: Derivation) -> Key<XPrv, Q> {
        let derivation_scheme = self.derivation_scheme;
        let key = self.key.derive(derivation_scheme, *derivation);
        let path = self.path.append_unchecked(derivation).coerce_unchecked();
        Key {
            key,
            path,
            derivation_scheme,
        }
    }

    /// derive the private key against the given derivation index and scheme
    ///
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub(crate) fn derive_path_unchecked<'a, Q, I>(&'a self, derivation_path: I) -> Key<XPrv, Q>
    where
        I: IntoIterator<Item = &'a Derivation>,
    {
        let derivation_scheme = self.derivation_scheme;

        let mut key = self.key.clone();
        let mut path = self.path.clone().coerce_unchecked::<Q>();

        for derivation in derivation_path {
            key = key.derive(derivation_scheme, **derivation);
            path = path.append_unchecked(*derivation);
        }

        Key {
            key,
            path,
            derivation_scheme,
        }
    }
}

impl<P> Key<XPub, P> {
    /// verify the signature with the public key for the given message
    #[inline]
    pub fn verify<T, B>(&self, message: B, signature: &Signature<T>) -> bool
    where
        B: AsRef<[u8]>,
    {
        self.key.verify(message.as_ref(), signature)
    }

    /// get the public key content without revealing the chaincode.
    #[inline]
    pub fn public_key_slice(&self) -> &[u8] {
        self.key.public_key_slice()
    }

    pub fn public_key(&self) -> &XPub {
        &self.key
    }

    pub fn pk(&self) -> chain_crypto::PublicKey<chain_crypto::Ed25519> {
        if let Ok(pk) = chain_crypto::PublicKey::from_binary(self.public_key_slice()) {
            pk
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    /// derive the private key against the given derivation index and scheme
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub(crate) fn derive_unchecked<Q>(&self, derivation: SoftDerivation) -> Key<XPub, Q> {
        let derivation_scheme = self.derivation_scheme;
        let key = if let Ok(key) = self.key.derive(derivation_scheme, *derivation) {
            key
        } else {
            // cannot happen because we already enforced the derivation index
            // to be a soft derivation.
            unsafe { std::hint::unreachable_unchecked() }
        };
        let path = self
            .path
            .append_unchecked(derivation.into())
            .coerce_unchecked();
        Key {
            key,
            path,
            derivation_scheme,
        }
    }
}

impl<P> Key<chain_crypto::SecretKey<chain_crypto::Ed25519Extended>, P> {
    /// create a signature for the given message and associate the given type `T`
    /// to the signature type.
    ///
    #[inline]
    pub fn sign<T, B>(&self, message: B) -> Signature<T>
    where
        B: AsRef<[u8]>,
    {
        Signature::from_slice(self.key.sign(&message.as_ref()).as_ref()).unwrap()
    }

    pub fn pk(&self) -> chain_crypto::PublicKey<chain_crypto::Ed25519> {
        self.key.to_public()
    }
}

impl<'a, P, Q> Iterator for KeyRange<'a, XPub, SoftDerivationRange, P, Q> {
    type Item = Key<XPub, Q>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|next| self.key.derive_unchecked(next))
    }
}

impl<'a, P, Q> Iterator for KeyRange<'a, XPrv, DerivationRange, P, Q> {
    type Item = Key<XPrv, Q>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|next| self.key.derive_unchecked(next))
    }
}

impl<K, P> AsRef<K> for Key<K, P> {
    fn as_ref(&self) -> &K {
        &self.key
    }
}

impl<K: Clone, P> Clone for Key<K, P> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            path: self.path.clone(),
            derivation_scheme: self.derivation_scheme,
        }
    }
}

impl<K, P> Debug for Key<K, P>
where
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!(
            "Key<{}, {}>",
            std::any::type_name::<K>(),
            std::any::type_name::<P>()
        ))
        .field("path", &self.path.to_string())
        .field("key", &self.key)
        .field("scheme", &self.derivation_scheme)
        .finish()
    }
}

impl<P> Display for Key<XPrv, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "<private-key> ({path} - {scheme:?})",
            path = self.path,
            scheme = self.derivation_scheme
        ))
    }
}

impl<P> Display for Key<XPub, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{key} ({path} - {scheme:?})",
            key = self.key,
            path = self.path,
            scheme = self.derivation_scheme,
        ))
    }
}
