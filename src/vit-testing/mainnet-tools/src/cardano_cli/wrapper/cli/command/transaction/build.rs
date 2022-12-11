use crate::cardano_cli::wrapper::utils::CommandExt;
use snapshot_trigger_service::config::NetworkType;
use std::path::Path;
use std::process::Command;
use tracing::debug;

pub struct Builder {
    command: Command,
}

impl Builder {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn network(mut self, network: NetworkType) -> Self {
        self.command.arg_network(network);
        self
    }

    pub fn tx_in(mut self, tx_in: String) -> Self {
        self.command.arg("--tx-in").arg(tx_in);
        self
    }

    pub fn change_address(mut self, change_address: String) -> Self {
        self.command.arg("--change-address").arg(change_address);
        self
    }

    pub fn certificate_file<P: AsRef<Path>>(mut self, certificate_file: P) -> Self {
        self.command
            .arg("--certificate-file")
            .arg(certificate_file.as_ref());
        self
    }

    pub fn protocol_params_file<P: AsRef<Path>>(mut self, protocol_params_file: P) -> Self {
        self.command
            .arg("--protocol-params-file")
            .arg(protocol_params_file.as_ref());
        self
    }

    pub fn out_file<P: AsRef<Path>>(mut self, out_file: P) -> Self {
        self.command.arg("--out-file").arg(out_file.as_ref());
        self
    }

    pub fn witness_override(mut self, witness_override: u32) -> Self {
        self.command
            .arg("--witness-override")
            .arg(witness_override.to_string());
        self
    }

    pub fn build(self) -> Command {
        debug!("Cardano Cli - transaction build: {:?}", self.command);
        self.command
    }
}
