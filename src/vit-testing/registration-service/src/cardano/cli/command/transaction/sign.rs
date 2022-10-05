use crate::config::NetworkType;
use crate::utils::CommandExt;
use std::path::Path;
use std::process::Command;

pub struct TransactionSignCommand {
    command: Command,
}

impl TransactionSignCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn tx_body_file<P: AsRef<Path>>(mut self, tx_body_file: P) -> Self {
        self.command
            .arg("--tx-body-file")
            .arg(tx_body_file.as_ref());
        self
    }

    pub fn signing_key_file<P: AsRef<Path>>(mut self, signing_key_file: P) -> Self {
        self.command
            .arg("--signing-key-file")
            .arg(signing_key_file.as_ref());
        self
    }

    pub fn network(mut self, network: NetworkType) -> Self {
        self.command.arg_network(network);
        self
    }

    pub fn out_file<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.command.arg("--out-file").arg(output.as_ref());
        self
    }

    pub fn build(self) -> Command {
        println!("Cardano Cli - transaction sign: {:?}", self.command);
        self.command
    }
}
