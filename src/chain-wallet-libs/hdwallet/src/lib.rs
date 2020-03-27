pub mod account;
pub mod bip44;
mod key;
pub mod keygen;
mod recovering;
pub mod rindex;

pub use self::{key::Key, recovering::RecoveringError};

pub struct Wallet {
    legacy_wallet: Option<rindex::Wallet>,
    wallet: Option<bip44::Wallet>,
    account: account::Account,
}

// check of the user want to recover daedalus wallet too?
// does the user want to recover a bip44 wallet too?
// recover the account
#[derive(Default)]
pub struct WalletRecoverer {
    legacy_wallet: Option<rindex::Wallet>,
    wallet: Option<bip44::Wallet>,
    account: Option<account::Account>,
}

impl WalletRecoverer {
    /// start recovering funds
    pub fn new() -> Self {
        Self::default()
    }

    /// recover the ethereum style account from the the given mnemonics and mnemonics password
    /// 
    /// this is the recommended method and is mandatory action to do.
    /// the bip39 mechanism is stronger as it provide brute force protections
    /// using HMAC SHA512 4096 iterations.
    /// 
    /// also it allows plausible deniability via the mnemonic password.
    pub fn account<D>(
        &mut self,
        dic: &D,
        mnemonics_phrase: &str,
        password: &[u8]
    ) -> Result<&mut Self, RecoveringError>
    where
        D: bip39::dictionary::Language,
    {
        let mnemonics = bip39::Mnemonics::from_string(dic, mnemonics_phrase)?;
        let entropy = bip39::Entropy::from_mnemonics(&mnemonics)?;

        let account = account::Account::new(&entropy, password);

        self.account = Some(account);
        Ok(self)
    }

    /// recover a wallet from reboot byron era. This is compatible with legacy byron
    /// era yoroi wallet too. This is interesting to recover funds from legacy byron yoroi
    /// wallet or from the byron reboot (daedalus or yoroi) wallets
    /// 
    /// However no new addresses will be generated from this wallet. This is mainly
    /// for security considerations.
    /// 
    /// TODO: find how the byron reboot wallets generated group keys (if needed at all)
    pub fn daedalus_utxo_wallet<D>(
        &mut self,
        dic: &D,
        mnemonics_phrase: &str,
    ) -> Result<&mut Self, RecoveringError>
    where
        D: bip39::dictionary::Language,
    {
        let modern_key = recovering::from_bip39_mnemonics(
            ed25519_bip32::DerivationScheme::V2,
            dic,
            mnemonics_phrase,
            &[]
        )?;
        let root = bip44::Root::from_root_key(modern_key);

        let wallet = bip44::Wallet::new(root);

        self.wallet = Some(wallet);

        Ok(self)
    }
   
    /// recover a legacy built wallet. This is interesting to recover funds from the
    /// legacy byron.
    /// 
    /// However no new addresses will be generated from this wallet. This is mainly
    /// for security considerations.
    /// 
    pub fn legacy_wallet<D>(
        &mut self,
        dic: &D,
        mnemonics_phrase: &str,
    ) -> Result<&mut Self, RecoveringError>
    where
        D: bip39::dictionary::Language,
    {
        let legacy_key = recovering::from_daedalus_mnemonics(
            ed25519_bip32::DerivationScheme::V1,
            dic,
            mnemonics_phrase
        )?;

        let legacy_wallet = rindex::Wallet::from_root_key(legacy_key);

        self.legacy_wallet = Some(legacy_wallet);

        Ok(self)
    }

    pub fn finish(self) -> Wallet {
        if let Some(account) = self.account {
            Wallet {
                legacy_wallet: self.legacy_wallet,
                wallet: self.wallet,
                account,
            }
        } else {
            panic!("account is missing yet it is mandatory")
        }
    }
}
