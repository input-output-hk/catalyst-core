use crate::{bip44::COIN_TYPE, Key};
use chain_path_derivation::{
    bip44::{self, Bip44, CoinType},
    HardDerivation,
};
use ed25519_bip32::{DerivationScheme, XPrv};

/// a key at the root level of the BIP44 based wallet
pub struct Root<K> {
    cached_key: Key<K, Bip44<CoinType>>,
}

impl<K> Root<K> {
    /// load the root key from the given Key and DerivationScheme
    ///
    /// Here it is expected that K has been derived already on the 2 first
    /// levels of the bip44 for Cardano Ada `m/'44/'1815`.
    ///
    pub fn from_cached_key(cached_key: K, derivation_scheme: DerivationScheme) -> Self {
        let cached_key = Key::new_unchecked(
            cached_key,
            bip44::new().purpose().coin_type(COIN_TYPE),
            derivation_scheme,
        );
        Self { cached_key }
    }

    pub(crate) fn cached_key(&self) -> &Key<K, Bip44<CoinType>> {
        &self.cached_key
    }
}

impl Root<XPrv> {
    /// create the wallet root from the given private key and derivation scheme
    ///
    /// this function assume no derivation happened on `root_key` and will apply
    /// the necessary derivation internally.
    pub fn from_root_key(root_key: Key<XPrv, Bip44<bip44::Root>>) -> Self {
        let path = bip44::new().purpose().coin_type(COIN_TYPE);

        let cached_key = root_key.derive_path_unchecked(path.iter());

        Self { cached_key }
    }

    pub(crate) fn derive_account(
        &self,
        derivation: HardDerivation,
    ) -> Key<XPrv, Bip44<bip44::Account>> {
        self.cached_key().derive_unchecked(derivation.into())
    }
}
