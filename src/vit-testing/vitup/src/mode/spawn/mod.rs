mod interactive;
mod monitor;
mod service;
mod standard;

use crate::builders::{FOLLOWER, LEADER_1, LEADER_2, LEADER_3};
use crate::config::{mode::Mode, Config};
use crate::mode::standard::{ValidVotingTemplateGenerator, WalletProxySpawnParams};
use crate::Result;
use hersir::config::{SessionSettings, SpawnParams};
use jormungandr_automation::jormungandr::PersistenceMode;
use jormungandr_lib::interfaces::Mempool;
use std::path::Path;
use std::path::PathBuf;
use valgrind::Protocol;

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
    protocol: Protocol,
    session_settings: SessionSettings,
    version: String,
    working_directory: PathBuf,
}

impl NetworkSpawnParams {
    pub fn new<P: AsRef<Path>>(
        endpoint: String,
        protocol: Protocol,
        session_settings: SessionSettings,
        token: Option<String>,
        version: String,
        working_directory: P,
    ) -> Self {
        Self {
            token,
            endpoint,
            protocol,
            session_settings,
            version,
            working_directory: working_directory.as_ref().to_path_buf(),
        }
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
            self.passive_node(FOLLOWER),
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
            .mempool(Mempool {
                pool_max_entries: 1_000_000.into(),
                ..Default::default()
            })
    }

    pub fn proxy_params(&self) -> WalletProxySpawnParams {
        let mut params = WalletProxySpawnParams::new(FOLLOWER.to_lowercase());
        params
            .with_base_address(self.endpoint.clone())
            .with_protocol(self.protocol.clone());
        params
    }
}
