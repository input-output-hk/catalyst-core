use crate::config::{Configuration, NetworkType};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Default, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    pub fn with_port(mut self, port: u16) -> Self {
        self.configuration.inner.address.set_port(port);
        self
    }

    pub fn with_result_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.configuration.inner.result_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn with_tmp_result_dir(self, tmp: &TempDir) -> Self {
        self.with_result_dir(tmp.child("registration_result").path())
    }

    pub fn with_cardano_cli<P: AsRef<Path>>(mut self, cardano_cli_bin: P) -> Self {
        self.configuration.cardano_cli = cardano_cli_bin.as_ref().to_path_buf();
        self
    }

    pub fn with_jcli<P: AsRef<Path>>(mut self, jcli_bin: P) -> Self {
        self.configuration.jcli = jcli_bin.as_ref().to_path_buf();
        self
    }

    pub fn with_voter_registration<P: AsRef<Path>>(mut self, voter_registration_bin: P) -> Self {
        self.configuration.voter_registration = voter_registration_bin.as_ref().to_path_buf();
        self
    }

    pub fn with_network(mut self, network: NetworkType) -> Self {
        self.configuration.network = network;
        self
    }

    pub fn build(self) -> Configuration {
        self.configuration
    }
}
