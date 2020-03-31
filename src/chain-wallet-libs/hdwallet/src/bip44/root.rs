use crate::{bip44::COIN_TYPE, keygen, Key};
use bip39;
use chain_path_derivation::{
    bip44::{self, Bip44, CoinType},
    AnyScheme, HardDerivation,
};
use ed25519_bip32::{DerivationScheme, XPrv, XPRV_SIZE};

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
            bip44::new().coin_type(COIN_TYPE),
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
    pub fn from_root_key(root_key: Key<XPrv, AnyScheme>) -> Self {
        let path = bip44::new().coin_type(COIN_TYPE);

        let cached_key = root_key.derive_path_unchecked(path.iter());

        Self { cached_key }
    }

    /// helper to create a wallet from BIP39 Seed
    ///
    /// We assume the [`MnemonicString`](../../bip/bip39/struct.MnemonicString.html)
    /// so we don't have to handle error in this constructor.
    ///
    /// Prefer `from_entropy` unless BIP39 seed generation compatibility is needed.
    pub fn from_bip39_seed(seed: &bip39::Seed, derivation_scheme: DerivationScheme) -> Self {
        let mut sk = [0; 32];
        sk.clone_from_slice(&seed.as_ref()[0..32]);
        let mut cc = [0; 32];
        cc.clone_from_slice(&seed.as_ref()[32..64]);
        let xprv = XPrv::from_nonextended_force(&sk, &cc);

        let key = Key::new_unchecked(xprv, Default::default(), derivation_scheme);

        Self::from_root_key(key)
    }

    /// helper to create a wallet from BIP39 mnemonics
    ///
    /// We assume the [`MnemonicString`](../../bip/bip39/struct.MnemonicString.html)
    /// so we don't have to handle error in this constructor.
    ///
    /// Prefer `from_entropy` unless BIP39 seed generation compatibility is needed.
    pub fn from_bip39_mnemonics<P>(
        mnemonics_phrase: &bip39::MnemonicString,
        password: P,
        derivation_scheme: DerivationScheme,
    ) -> Self
    where
        P: AsRef<[u8]>,
    {
        let seed = bip39::Seed::from_mnemonic_string(mnemonics_phrase, password.as_ref());

        Self::from_bip39_seed(&seed, derivation_scheme)
    }

    /// Create a new wallet from a root entropy
    ///
    /// This is the recommended method to create a wallet from initial generated value.
    ///
    /// Note this method, doesn't put the bip39 dictionary used in the cryptographic data,
    /// hence the way the mnemonics are displayed is independent of the language chosen.
    pub fn from_entropy(
        entropy: &bip39::Entropy,
        password: &[u8],
        derivation_scheme: DerivationScheme,
    ) -> Self {
        let mut seed = [0u8; XPRV_SIZE];
        keygen::generate_seed(entropy, password, &mut seed);
        let xprv = XPrv::normalize_bytes_force3rd(seed);

        let key = Key::new_unchecked(xprv, Default::default(), derivation_scheme);

        Self::from_root_key(key)
    }

    pub(crate) fn derive_account(
        &self,
        derivation: HardDerivation,
    ) -> Key<XPrv, Bip44<bip44::Account>> {
        self.cached_key().derive_unchecked(derivation.into())
    }
}
