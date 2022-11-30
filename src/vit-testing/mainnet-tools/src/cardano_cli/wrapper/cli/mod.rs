use crate::cardano_cli::wrapper::cli::api::{Address, Query, StakeAddress, Transaction};
use crate::cardano_cli::wrapper::cli::command::Root;
use std::path::PathBuf;
use std::process::Command;

pub mod api;
pub mod command;

/// Cardano Cli wrapper which allows to use cardano cli as Rust library.
/// It requires real `CardanoCli` to be installed on environment
pub struct Api {
    cardano_cli: PathBuf,
}

impl Api {
    /// Creates new Api object based on path to cardano cli
    #[must_use]
    pub fn new(cardano_cli: PathBuf) -> Self {
        Self { cardano_cli }
    }

    /// Address related sub commands
    #[must_use]
    pub fn address(&self) -> Address {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = Root::new(command);
        Address::new(cardano_cli_command.address())
    }

    /// Stake address related sub commands
    #[must_use]
    pub fn stake_address(&self) -> StakeAddress {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = Root::new(command);
        StakeAddress::new(cardano_cli_command.stake_address())
    }

    /// Transaction related sub commands
    #[must_use]
    pub fn transaction(&self) -> Transaction {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = Root::new(command);
        Transaction::new(cardano_cli_command.transaction())
    }

    /// Query related sub commands
    #[must_use]
    pub fn query(&self) -> Query {
        let command = Command::new(self.cardano_cli.clone());
        let cardano_cli_command = Root::new(command);
        Query::new(cardano_cli_command.query())
    }
}
