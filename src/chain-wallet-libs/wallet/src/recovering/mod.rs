//! module for all the recovering mechanism around the cardano blockchains

mod paperwallet;

use crate::{account::Wallet, keygen, scheme as wallet, Password};
use chain_crypto::{Ed25519Extended, SecretKey};
use chain_impl_mockchain::legacy::OldAddress;
use chain_path_derivation::{
    bip44::{self, Bip44},
    AnyScheme,
};
use ed25519_bip32::{self, DerivationScheme, XPrv, XPRV_SIZE};
use hdkeygen::Key;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("The wallet scheme does not require a password")]
    SchemeDoesNotRequirePassword,

    #[error("Missing entropy, either missing the mnemonics or need to generate a new wallet")]
    MissingEntropy,
    #[error("Tried to recover same utxo more than once, either the function was called twice or the block is malformed")]
    DuplicatedUtxo,
}

pub struct RecoveryBuilder {
    entropy: Option<bip39::Entropy>,
    password: Option<Password>,
    free_keys: Vec<SecretKey<Ed25519Extended>>,
    account: Option<AccountFrom>,
}

enum AccountFrom {
    Seed(hdkeygen::account::Seed),
    SecretKey(SecretKey<Ed25519Extended>),
}

impl RecoveryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// instead of recovering from mnemonics recover from a paperwallet
    ///
    pub fn paperwallet(
        self,
        password: impl AsRef<[u8]>,
        input: impl AsRef<[u8]>,
    ) -> Result<Self, bip39::Error> {
        let entropy = paperwallet::unscramble(password.as_ref(), input.as_ref());

        let entropy = bip39::Entropy::from_slice(&entropy)?;

        Ok(self.entropy(entropy))
    }

    pub fn mnemonics<D>(self, dic: &D, mnemonics: impl AsRef<str>) -> Result<Self, bip39::Error>
    where
        D: bip39::dictionary::Language,
    {
        let entropy = if let Some(entropy) = paperwallet::daedalus_paperwallet(mnemonics.as_ref())?
        {
            entropy
        } else {
            let mnemonics = bip39::Mnemonics::from_string(dic, mnemonics.as_ref())?;
            bip39::Entropy::from_mnemonics(&mnemonics)?
        };

        Ok(self.entropy(entropy))
    }

    pub fn entropy(self, entropy: bip39::Entropy) -> Self {
        Self {
            entropy: Some(entropy),
            ..self
        }
    }

    pub fn password(self, password: Password) -> Self {
        Self {
            password: Some(password),
            ..self
        }
    }

    pub fn account_seed(self, seed: hdkeygen::account::Seed) -> Self {
        Self {
            account: Some(AccountFrom::Seed(seed)),
            ..self
        }
    }

    pub fn account_secret_key(self, key: SecretKey<Ed25519Extended>) -> Self {
        Self {
            account: Some(AccountFrom::SecretKey(key)),
            ..self
        }
    }

    pub fn add_key(mut self, key: SecretKey<Ed25519Extended>) -> Self {
        self.free_keys.push(key);
        self
    }

    pub fn build_yoroi(&self) -> Result<wallet::bip44::Wallet<OldAddress>, RecoveryError> {
        let entropy = self.entropy.clone().ok_or(RecoveryError::MissingEntropy)?;
        let password = self.password.clone().unwrap_or_default();

        let key = from_bip39_entropy(entropy, password, ed25519_bip32::DerivationScheme::V2);

        let root: Key<XPrv, Bip44<bip44::Root>> = key.coerce_unchecked();
        let key = root.bip44().cardano();

        Ok(wallet::bip44::Wallet::<OldAddress>::from_root_key(key))
    }

    pub fn build_wallet(&self) -> Result<Wallet, RecoveryError> {
        let wallet = match &self.account {
            Some(AccountFrom::SecretKey(key)) => Wallet::new_from_key(key.clone()),
            Some(AccountFrom::Seed(seed)) => Wallet::new_from_seed(*seed),
            None => {
                let entropy = self.entropy.clone().ok_or(RecoveryError::MissingEntropy)?;
                let password = self.password.clone().unwrap_or_default();

                let mut seed = [0u8; hdkeygen::account::SEED_LENGTH];
                keygen::generate_seed(&entropy, password.as_ref(), &mut seed);

                Wallet::new_from_seed(seed)
            }
        };
        Ok(wallet)
    }

    pub fn build_free_utxos(&self) -> Result<wallet::freeutxo::Wallet, RecoveryError> {
        Ok(wallet::freeutxo::Wallet::from_keys(self.free_keys.clone()))
    }

    #[cfg(test)]
    fn to_mnemonics_string(&self) -> Option<String> {
        let s = self.entropy.as_ref()?;
        let mnemonics = s.to_mnemonics();
        let mnemonics = mnemonics.to_string(&bip39::dictionary::ENGLISH);
        Some(mnemonics.to_string())
    }
}

/// method to recover the private key from bip39 mnemonics
///
/// this is the method used in yoroiwallet.com
fn from_bip39_entropy(
    entropy: bip39::Entropy,
    password: impl AsRef<[u8]>,
    derivation_scheme: DerivationScheme,
) -> Key<XPrv, AnyScheme> {
    let mut seed = [0u8; XPRV_SIZE];
    keygen::generate_seed(&entropy, password.as_ref(), &mut seed);
    let xprv = XPrv::normalize_bytes_force3rd(seed);

    Key::new_unchecked(xprv, Default::default(), derivation_scheme)
}

impl Default for RecoveryBuilder {
    fn default() -> RecoveryBuilder {
        RecoveryBuilder {
            entropy: Default::default(),
            password: Default::default(),
            free_keys: Vec::<SecretKey<Ed25519Extended>>::new(),
            account: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MNEMONICS1: &str =
        "tired owner misery large dream glad upset welcome shuffle eagle pulp time";
    const ADDRESSES1: &[&str] = &[
        "DdzFFzCqrhsktawSMCWJJy3Dpp9BCjYPVecgsMb5U2G7d1ErUUmwSZvfSY3Yjn5njNadfwvebpVNS5cD4acEKSQih2sR76wx2kF4oLXT",
        "DdzFFzCqrhsg7eQHEfFE7cH7bKzyyUEKSoSiTmQtxAGnAeCW3pC2LXyxaT8T5sWH4zUjfjffik6p9VdXvRfwJgipU3tgzXhKkMDLt1hR",
        "DdzFFzCqrhsw7G6njwb8FTBxVCh9GtB7RFvvz7KPNkHxeHtDwAPT2Y6QLDLxVCu7NNUQmwpAfgG5ZeGQkoWjrkbHPUeU9wzG3YFpohse",
    ];

    const MNEMONICS2: &str =
        "edge club wrap where juice nephew whip entry cover bullet cause jeans";
    const ADDRESSES2: &[&str] = &[
        "DdzFFzCqrhsf2sWcZLzXhyLoLZcmw3Zf3UcJ2ozG1EKTwQ6wBY1wMG1tkXtPvEgvE5PKUFmoyzkP8BL4BwLmXuehjRHJtnPj73E5RPMx",
        "DdzFFzCqrhsogWSfcp4Dq9W1bcMzt86276PbDfzAKZxDhi3g6w6fRu6zYMT36uG8p3j8bCgsx4frkB3QH8m8ubUhAKRG5c8SLnGVTBh9",
        "DdzFFzCqrhtDFbFvtrm3hhHuWUPY9ozkCW5JzuL4TcrXKMruWCrCSRzpc4mkWBUugPAGLesJv3ert9BH1cQJqXq2f4UN83WP5AZZN4jQ",
        "sxtitePxjp57M5Vf1uXXvYzTBn3AXrLriV1AXUvEwAdbQckZyh9erD1fBMy7168gkoqWq9jgMHjgW62ZrAcxqxP8Y5",
        "sxtitePxjp5ewhRtrqYCu1h8BnWz2GCRbT26FFuvhetcaWfw1rZNX4vpQpXqiygvJBGAWsjLzrTp3EzCZ6cYK6A2YT",
        "sxtitePxjp5Y7GQre2hj7LPAnZp7F49KxE6Cg1huwTzjWbfW2Jd7hSgSqsbMzESs8aQC44ng1LJdnLKqiou4m4gGy8",
    ];

    const MNEMONICS3: &str = "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";
    const ADDRESSES3: &[&str] = &[
        "Ae2tdPwUPEZ8og5u4WF5rmSyme5Gvp8RYiLM2u7Vm8CyDQzLN3VYTN895Wk",
        "Ae2tdPwUPEZEAjEsQsCtBMkLKANxQUEvzLkumPWWYugLeXcgkeMCDH1gnuL",
    ];

    #[test]
    fn recover_yoroi_utxo_twice_fails() {
        use chain_impl_mockchain::{
            fragment::Fragment,
            legacy::{OldAddress, UtxoDeclaration},
            value::Value,
        };

        let mut wallet = RecoveryBuilder::new()
            .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS3)
            .unwrap()
            .build_yoroi()
            .unwrap();

        let address: OldAddress = ADDRESSES3[0].parse().unwrap();
        assert!(wallet.check_address(&address).is_some());

        let fragment_value = Value(10);
        let fragment = Fragment::OldUtxoDeclaration(UtxoDeclaration {
            addrs: vec![(address, fragment_value)],
        });
        let fragment_id = fragment.hash();

        assert_eq!(wallet.unconfirmed_value(), None);
        assert!(wallet.check_fragment(&fragment_id, &fragment));
        assert_eq!(wallet.unconfirmed_value(), Some(fragment_value));
        assert!(wallet.check_fragment(&fragment_id, &fragment));
        assert_eq!(wallet.unconfirmed_value(), Some(fragment_value));
    }

    #[test]
    #[ignore]
    fn recover_yoroi_paperwallet() {
        /*
            XXX: yoroi paper wallet are excluded for now

        const WALLET: &str = "attend wink add online sample oyster guard glass host gap business music faith riot tortoise";
        const PWD: &str = "PaperWalletPaperWallet";
        const PAPERWALLET: &str = "weasel use dynamic shock food bleak swarm owner trick also flight flower uncover slim fuel crisp hockey reunion lemon badge orient";

        let builder = RecoveryBuilder::new()
            .mnemonics(&bip39::dictionary::ENGLISH, PAPERWALLET)
            .unwrap()
            .password(Password::from(PWD.to_owned()))
            .build_yoroi();
        */
    }
}
