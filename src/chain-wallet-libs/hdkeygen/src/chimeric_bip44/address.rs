use crate::{chimeric_bip44::COIN_TYPE, Key};
use chain_path_derivation::{
    chimeric_bip44::{self, ChimericBip44},
    DerivationPath,
};
use ed25519_bip32::{DerivationScheme, XPrv, XPub};
use std::ops::Deref;

pub struct Address<K> {
    key: Key<K, ChimericBip44<chimeric_bip44::Address>>,
}

impl Address<XPrv> {
    pub fn public(&self) -> Address<XPub> {
        Address {
            key: self.key.public(),
        }
    }
}

impl<K> Address<K> {
    pub fn new(key: Key<K, ChimericBip44<chimeric_bip44::Address>>) -> Self {
        Self { key }
    }

    /// load the account key from the given Key, DerivationScheme and index
    ///
    /// Here it is expected that K has been derived already on the 5 first
    /// levels of the bip44 for Cardano Ada coin type
    ///
    /// # panics
    ///
    /// This function will panic if path's coin_type is not Cardano ADA
    /// coin type.
    pub fn from_key(
        key: K,
        derivation_scheme: DerivationScheme,
        path: DerivationPath<ChimericBip44<chimeric_bip44::Address>>,
    ) -> Self {
        assert_eq!(
            path.coin_type(),
            COIN_TYPE,
            "Expecting Cardano ADA coin type"
        );

        let key = Key::new_unchecked(key, path, derivation_scheme);
        Self::new(key)
    }

    pub fn key(&self) -> &Key<K, ChimericBip44<chimeric_bip44::Address>> {
        &self.key
    }
}

impl<K> Deref for Address<K> {
    type Target = Key<K, ChimericBip44<chimeric_bip44::Address>>;
    fn deref(&self) -> &Self::Target {
        self.key()
    }
}

impl<K: Clone> Clone for Address<K> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
        }
    }
}
