use crate::cardano::cli::api::{Address, Query, StakeAddress, Transaction};
use crate::cardano::cli::command::CardanoCliCommand;
use std::path::PathBuf;
use std::process::Command;

pub mod api;
pub mod command;

pub struct CardanoCli {
    cardano_cli: PathBuf,
}

impl CardanoCli {
    pub fn new(cardano_cli: PathBuf) -> Self {
        Self { cardano_cli }
    }

    pub fn address(&self) -> Address {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = CardanoCliCommand::new(command);
        Address::new(cardano_cli_command.address())
    }

    pub fn stake_address(&self) -> StakeAddress {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = CardanoCliCommand::new(command);
        StakeAddress::new(cardano_cli_command.stake_address())
    }

    pub fn transaction(&self) -> Transaction {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = CardanoCliCommand::new(command);
        Transaction::new(cardano_cli_command.transaction())
    }

    pub fn query(&self) -> Query {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = CardanoCliCommand::new(command);
        Query::new(cardano_cli_command.query())
    }
}
