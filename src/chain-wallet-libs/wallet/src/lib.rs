mod blockchain;
mod keygen;
mod password;
mod recovering;
pub mod scheme;
mod store;
pub mod transaction;

pub use self::{
    blockchain::Settings,
    password::{Password, ScrubbedBytes},
    recovering::{RecoveryBuilder, RecoveryError},
    store::{StateIter, States, Status, UtxoGroup, UtxoStore},
    transaction::{AccountWitnessBuilder, TransactionBuilder, WitnessBuilder},
};
use chain_impl_mockchain::{transaction::UnspecifiedAccountIdentifier, value::Value};
use hdkeygen::account::Account;
pub use hdkeygen::account::AccountId;

pub struct Wallet {
    account: Account,
    value: Value,
    committed_amount: Value,
}

impl Wallet {
    pub fn account_id(&self) -> AccountId {
        self.account.account_id()
    }

    pub fn remove(&mut self, id: UnspecifiedAccountIdentifier, value: Value) {
        let id = id.as_ref();
        if self.account.account_id().as_ref() == id {
            self.committed_amount = self
                .committed_amount
                .checked_sub(value)
                .unwrap_or_else(|_| Value::zero());
            self.value = self
                .value
                .checked_sub(value)
                .unwrap_or_else(|_| Value::zero());
        }
    }

    /// set the state counter so we can sync with the blockchain and the
    /// local state
    ///
    /// TODO: some handling to provide information if needed:
    ///
    /// - [ ] check the counter is not regressing?
    /// - [ ] check that there is continuity?
    ///
    pub fn update_state(&mut self, value: Value, counter: u32) {
        self.value = value;
        self.committed_amount = Value::zero();
        self.account.set_counter(counter);
    }

    pub fn value(&self) -> Value {
        self.value
    }

    pub fn committed_amount(&self) -> Value {
        self.committed_amount
    }

    pub fn witness_builder(&self) -> transaction::AccountWitnessBuilder {
        transaction::AccountWitnessBuilder(self.account.clone())
    }

    fn current_value(&self) -> Value {
        (self.value() - self.committed_amount()).unwrap_or_else(|_| Value::zero())
    }
}
