mod blockchain;
mod keygen;
mod password;
mod recovering;

pub use self::{
    blockchain::Settings,
    password::{Password, ScrubbedBytes},
    recovering::{Dump, RecoveringDaedalus, RecoveringIcarus, RecoveryBuilder, RecoveryError},
};
use hdkeygen::account::{Account, AccountId};

pub struct Wallet {
    account: Account,
}

impl Wallet {
    pub fn account_id(&self) -> AccountId {
        self.account.account_id()
    }
}
