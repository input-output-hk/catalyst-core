mod account;
mod address;
mod chimeric_account;
mod root;

pub use self::{account::Account, address::Address, chimeric_account::ChimericAccount, root::Root};
use chain_path_derivation::{Derivation, HardDerivation};
use ed25519_bip32::XPrv;

/// the derivation index for Cardano (Ada) Blockchain.
pub const COIN_TYPE: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0717));

/// wallet for Cardano, using BIP44 accounting model for UTxO
pub struct Wallet {
    root: Root<XPrv>,
}

impl Wallet {
    /// create a new Wallet with the given root key and **no account** yet.
    pub fn new(root: Root<XPrv>) -> Self {
        Self { root }
    }

    /// create an account for the given account ID
    ///
    /// This function does not check if the account is already valid or not.
    /// If the account does not exist it is then created and the account
    /// is then returned. Otherwise the existing account is returned.
    pub fn create_account<I>(&mut self, account_id: I) -> Account<XPrv>
    where
        I: Into<HardDerivation>,
    {
        let hd = account_id.into();

        let key = self.root.derive_account(hd);
        Account::new(key)
    }
}
