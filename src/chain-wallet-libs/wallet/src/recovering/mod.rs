//! module for all the recovering mechanism around the cardano blockchains

mod paperwallet;

use crate::{account::Wallet, keygen, scheme as wallet, Password};
use chain_crypto::{Ed25519Extended, SecretKey};
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
