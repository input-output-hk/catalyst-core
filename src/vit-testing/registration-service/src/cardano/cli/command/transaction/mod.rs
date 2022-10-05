mod build;
mod id;
mod sign;
mod submit;

use std::process::Command;

use crate::cardano::cli::command::transaction::build::TransactionBuildCommand;
pub use id::TransactionIdCommand;
pub use sign::TransactionSignCommand;
pub use submit::TransactionSubmitCommand;

pub struct TransactionCommand {
    command: Command,
}

impl TransactionCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn id(mut self) -> TransactionIdCommand {
        self.command.arg("txid");
        id::TransactionIdCommand::new(self.command)
    }

    pub fn sign(mut self) -> TransactionSignCommand {
        self.command.arg("sign");
        sign::TransactionSignCommand::new(self.command)
    }

    pub fn submit(mut self) -> TransactionSubmitCommand {
        self.command.arg("submit");
        TransactionSubmitCommand::new(self.command)
    }

    pub fn build(mut self) -> TransactionBuildCommand {
        self.command.arg("build");
        TransactionBuildCommand::new(self.command)
    }
}
