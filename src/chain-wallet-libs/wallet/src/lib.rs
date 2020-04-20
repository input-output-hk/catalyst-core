mod blockchain;
mod keygen;
mod password;
mod recovering;
pub mod transaction;

pub use self::{
    blockchain::Settings,
    password::{Password, ScrubbedBytes},
    recovering::{RecoveringDaedalus, RecoveringIcarus, RecoveryBuilder, RecoveryError},
};
use chain_impl_mockchain::{transaction::Input, value::Value};
use hdkeygen::account::{Account, AccountId};

pub struct Wallet {
    account: Account,

    value: Value,
    committed_amount: Value,
}

impl Wallet {
    pub fn account_id(&self) -> AccountId {
        self.account.account_id()
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub fn committed_amount(&self) -> Value {
        self.committed_amount.clone()
    }

    fn current_value(&self) -> Value {
        (self.value() - self.committed_amount()).unwrap_or(Value::zero())
    }
}
