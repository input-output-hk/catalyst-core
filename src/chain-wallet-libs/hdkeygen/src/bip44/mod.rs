mod account;
mod address;
mod root;

pub use self::{account::Account, address::Address, root::Root};
use chain_path_derivation::{Derivation, HardDerivation};
use ed25519_bip32::XPrv;
use std::collections::BTreeMap;

/// the derivation index for Cardano (Ada) Blockchain.
pub const COIN_TYPE: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0717));

/// wallet for Cardano, using BIP44 accounting model for UTxO
pub struct Wallet {
    root: Root<XPrv>,
    accounts: BTreeMap<HardDerivation, Account<XPrv>>,
}

impl Wallet {
    pub fn new_with(root: Root<XPrv>, accounts: BTreeMap<HardDerivation, Account<XPrv>>) -> Self {
        Self { root, accounts }
    }

    /// create a new Wallet with the given root key and **no account** yet.
    pub fn new(root: Root<XPrv>) -> Self {
        Self::new_with(root, BTreeMap::default())
    }

    /// create the given accounts, if the accounts already exists then nothing is changed
    ///
    pub fn create_accounts<I>(&mut self, accounts: I)
    where
        I: IntoIterator<Item = HardDerivation>,
    {
        for account in accounts {
            self.create_account(account);
        }
    }

    /// create an account for the given account ID
    ///
    /// This function does not check if the account is already valid or not.
    /// If the account does not exist it is then created and the account
    /// is then returned. Otherwise the existing account is returned.
    pub fn create_account<I>(&mut self, account_id: I) -> &Account<XPrv>
    where
        I: Into<HardDerivation>,
    {
        let hd = account_id.into();

        let key = self.root.derive_account(hd);
        let account = Account::new(key);

        self.accounts.entry(hd).or_insert(account)
    }
}
