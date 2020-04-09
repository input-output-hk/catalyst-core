use crate::{
    bip44::{Address, COIN_TYPE},
    Key,
};
use chain_path_derivation::{
    bip44::{self, Bip44},
    DerivationPath, HardDerivation, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::{DerivationScheme, XPrv, XPub};

pub struct Account<K> {
    key: Key<K, Bip44<bip44::Account>>,
}

pub struct AddressRange<K> {
    key: Key<K, Bip44<bip44::Change>>,
    range: SoftDerivationRange,
}

impl Account<XPrv> {
    pub fn public(&self) -> Account<XPub> {
        Account {
            key: self.key.public(),
        }
    }

    pub fn addresses(
        &self,
        change: SoftDerivation,
        range: SoftDerivationRange,
    ) -> AddressRange<XPrv> {
        let key = self.cached_key().derive_unchecked(change.into());
        AddressRange { key, range }
    }
}

impl Account<XPub> {
    pub fn addresses(
        &self,
        change: SoftDerivation,
        range: SoftDerivationRange,
    ) -> AddressRange<XPub> {
        let key = self.cached_key().derive_unchecked(change);
        AddressRange { key, range }
    }
}

impl<K> Account<K> {
    pub fn new(key: Key<K, Bip44<bip44::Account>>) -> Self {
        Self { key }
    }

    pub fn path(&self) -> &DerivationPath<Bip44<bip44::Account>> {
        self.key.path()
    }

    pub fn id(&self) -> HardDerivation {
        self.path().account()
    }

    /// load the account key from the given Key, DerivationScheme and index
    ///
    /// Here it is expected that K has been derived already on the 3 first
    /// levels of the bip44 for Cardano Ada `m/'44/'1815/'<derivation>`.
    ///
    pub fn from_key(
        key: K,
        derivation_scheme: DerivationScheme,
        derivation: HardDerivation,
    ) -> Self {
        let key = Key::new_unchecked(
            key,
            bip44::new()
                .purpose()
                .coin_type(COIN_TYPE)
                .account(derivation),
            derivation_scheme,
        );
        Self::new(key)
    }

    pub(crate) fn cached_key(&self) -> &Key<K, Bip44<bip44::Account>> {
        &self.key
    }
}

impl Iterator for AddressRange<XPub> {
    type Item = Address<XPub>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next);
        Some(Address::new(key))
    }
}

impl Iterator for AddressRange<XPrv> {
    type Item = Address<XPrv>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next.into());
        Some(Address::new(key))
    }
}

impl DoubleEndedIterator for AddressRange<XPub> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next);
        Some(Address::new(key))
    }
}

impl DoubleEndedIterator for AddressRange<XPrv> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next.into());
        Some(Address::new(key))
    }
}

impl<K> ExactSizeIterator for AddressRange<K>
where
    AddressRange<K>: Iterator,
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<K> std::iter::FusedIterator for AddressRange<K> where AddressRange<K>: Iterator {}

impl<K: Clone> Clone for Account<K> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
        }
    }
}
