mod address;
mod query;
mod stake_address;
mod transaction;

pub use address::Address;
pub use query::Query;
pub use stake_address::StakeAddress;
use std::process::Command;
pub use transaction::Transaction;

pub struct Root {
    command: Command,
}

impl Root {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn query(mut self) -> Query {
        self.command.arg("query");
        query::Query::new(self.command)
    }

    pub fn address(mut self) -> Address {
        self.command.arg("address");
        address::Address::new(self.command)
    }

    pub fn stake_address(mut self) -> StakeAddress {
        self.command.arg("stake-address");
        stake_address::StakeAddress::new(self.command)
    }

    pub fn transaction(mut self) -> Transaction {
        self.command.arg("transaction");
        transaction::Transaction::new(self.command)
    }
}
