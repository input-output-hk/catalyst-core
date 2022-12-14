use crate::cardano_cli::wrapper::utils::CommandExt;
use snapshot_trigger_service::config::NetworkType;
use std::path::Path;
use std::process::Command;

pub struct Query {
    command: Command,
}

impl Query {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn tip(mut self, network: NetworkType) -> Self {
        self.command.arg("tip").arg_network(network);
        self
    }

    pub fn utxo<S: Into<String>>(mut self, network: NetworkType, payment_address: S) -> Self {
        self.command
            .arg("utxo")
            .arg_network(network)
            .arg("--address")
            .arg(payment_address.into());
        self
    }

    pub fn protocol_parameters<P: AsRef<Path>>(
        mut self,
        network: NetworkType,
        out_file: P,
    ) -> Self {
        self.command
            .arg("protocol-parameters")
            .arg_network(network)
            .arg("--out-file")
            .arg(out_file.as_ref());
        self
    }

    pub fn build(self) -> Command {
        println!("Cardano Cli - query utxo: {:?}", self.command);
        self.command
    }
}
