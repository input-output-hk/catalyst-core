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

impl transaction::InputGenerator for Wallet {
    fn input_to_cover(&mut self, value: Value) -> Option<transaction::GeneratedInput> {
        if self.current_value() < value {
            None
        } else {
            let input = Input::from_account_public_key(self.account_id().into(), value);
            let witness_builder = transaction::WitnessBuilder::Account {
                account: self.account.clone(),
            };

            self.committed_amount = self.committed_amount.saturating_add(value);
            self.account.increase_counter(1);

            Some(transaction::GeneratedInput {
                input,
                witness_builder,
            })
        }
    }
}
