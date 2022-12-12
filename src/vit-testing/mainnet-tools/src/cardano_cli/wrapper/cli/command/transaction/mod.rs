#![allow(clippy::module_name_repetitions)]

mod build;
mod id;
mod sign;
mod submit;

use std::process::Command;
pub use id::Builder as IdCommandBuilder;
pub use sign::Builder as SignCommandBuilder;
pub use submit::Builder as SubmitCommandBuilder;
pub use build::Builder as TransactionCommandBuilder;

pub struct Transaction {
    command: Command,
}

impl Transaction {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn id(mut self) -> IdCommandBuilder {
        self.command.arg("txid");
        IdCommandBuilder::new(self.command)
    }

    pub fn sign(mut self) -> SignCommandBuilder {
        self.command.arg("sign");
        SignCommandBuilder::new(self.command)
    }

    pub fn submit(mut self) -> SubmitCommandBuilder {
        self.command.arg("submit");
        SubmitCommandBuilder::new(self.command)
    }

    pub fn build(mut self) -> TransactionCommandBuilder {
        self.command.arg("build");
        TransactionCommandBuilder::new(self.command)
    }
}
