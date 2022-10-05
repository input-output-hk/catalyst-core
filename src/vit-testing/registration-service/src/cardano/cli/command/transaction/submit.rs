use std::path::Path;
use std::process::Command;

pub struct TransactionSubmitCommand {
    command: Command,
}

impl TransactionSubmitCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn tx_file<P: AsRef<Path>>(mut self, tx_signed: P) -> Self {
        self.command.arg("--tx-file").arg(tx_signed.as_ref());
        self
    }

    pub fn build(self) -> Command {
        println!("Cardano Cli - transaction submit: {:?}", self.command);
        self.command
    }
}
