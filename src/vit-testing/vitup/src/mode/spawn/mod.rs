mod interactive;
mod monitor;
mod service;
mod standard;

use crate::builders::{LEADER_1, LEADER_2, LEADER_3, WALLET_NODE};
use crate::config::CertificatesBuilder;
use crate::config::{mode::Mode, Config};
use crate::mode::standard::{ValidVotingTemplateGenerator, WalletProxySpawnParams};
use crate::Result;
use hersir::builder::SpawnParams;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::PersistenceMode;
use std::path::Path;
use std::path::PathBuf;
use valgrind::Certs;

pub fn spawn_network(
    mode: Mode,
    network_spawn_params: NetworkSpawnParams,
    generator: &mut dyn ValidVotingTemplateGenerator,
    config: Config,
) -> Result<()> {
    match mode {
        Mode::Standard => standard::spawn_network(network_spawn_params, config, generator),
        Mode::Monitor => monitor::spawn_network(network_spawn_params, config, generator),
        Mode::Interactive => interactive::spawn_network(network_spawn_params, config, generator),
        Mode::Service => service::spawn_network(network_spawn_params, config, generator),
    }
}

#[derive(Debug, Clone)]
pub struct NetworkSpawnParams {
    token: Option<String>,
    endpoint: String,
    certs: Certs,
    session_settings: SessionSettings,
    version: String,
    working_directory: PathBuf,
}

impl NetworkSpawnParams {
    pub fn new<P: AsRef<Path>>(
        endpoint: String,
        parameters: &Config,
        session_settings: SessionSettings,
        token: Option<String>,
        working_directory: P,
    ) -> Result<Self> {
        let working_directory = working_directory.as_ref();

        Ok(Self {
            token,
            endpoint,
            certs: CertificatesBuilder::default().build(&working_directory)?,
            session_settings,
            version: parameters.version.clone(),
            working_directory: working_directory.to_path_buf(),
        })
    }
    pub fn session_settings(&self) -> SessionSettings {
        self.session_settings.clone()
    }

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn nodes_params(&self) -> Vec<SpawnParams> {
        vec![
            self.leader_node(LEADER_1),
            self.leader_node(LEADER_2),
            self.leader_node(LEADER_3),
            self.passive_node(WALLET_NODE),
        ]
    }

    fn leader_node(&self, alias: &str) -> SpawnParams {
        SpawnParams::new(alias)
            .leader()
            .persistence_mode(PersistenceMode::Persistent)
            .jormungandr(self.session_settings.jormungandr.clone())
    }

    fn passive_node(&self, alias: &str) -> SpawnParams {
        SpawnParams::new(alias)
            .passive()
            .persistence_mode(PersistenceMode::Persistent)
            .persistent_fragment_log(self.working_directory.clone().join("persistent_log"))
            .jormungandr(self.session_settings.jormungandr.clone())
    }

    pub fn proxy_params(&self) -> WalletProxySpawnParams {
        let mut params = WalletProxySpawnParams::new(WALLET_NODE);
        params
            .with_base_address(self.endpoint.clone())
            .with_certs(self.certs.clone());
        params
    }
}
