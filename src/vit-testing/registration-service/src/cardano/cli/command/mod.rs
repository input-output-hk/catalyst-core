mod address;
mod query;
mod stake_address;
mod transaction;

pub use address::AddressCommand;
pub use query::QueryCommand;
pub use stake_address::StakeAddressCommand;
use std::process::Command;
pub use transaction::{
    TransactionCommand, TransactionIdCommand, TransactionSignCommand, TransactionSubmitCommand,
};

pub struct CardanoCliCommand {
    command: Command,
}

impl CardanoCliCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn query(mut self) -> QueryCommand {
        self.command.arg("query");
        query::QueryCommand::new(self.command)
    }

    pub fn address(mut self) -> AddressCommand {
        self.command.arg("address");
        address::AddressCommand::new(self.command)
    }

    pub fn stake_address(mut self) -> StakeAddressCommand {
        self.command.arg("stake-address");
        stake_address::StakeAddressCommand::new(self.command)
    }

    pub fn transaction(mut self) -> TransactionCommand {
        self.command.arg("transaction");
        transaction::TransactionCommand::new(self.command)
    }
}
