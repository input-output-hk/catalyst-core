mod interactive;
mod monitor;
mod service;
mod standard;

use crate::builders::VitBackendSettingsBuilder;
use crate::builders::{LEADER_1, LEADER_2, LEADER_3, WALLET_NODE};
use crate::config::{mode::Mode, VitStartParameters};
use crate::vit_station::ValidVotingTemplateGenerator;
use crate::wallet::WalletProxySpawnParams;
use crate::Result;
use hersir::builder::SpawnParams;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::PersistenceMode;
use std::path::Path;
use std::path::PathBuf;
use valgrind::Protocol;

pub fn spawn_network(
    mode: Mode,
    session_settings: SessionSettings,
    network_spawn_params: NetworkSpawnParams,
    generator: &mut dyn ValidVotingTemplateGenerator,
    mut quick_setup: VitBackendSettingsBuilder,
) -> Result<()> {
    match mode {
        Mode::Standard => standard::spawn_network(
            session_settings,
            network_spawn_params,
            quick_setup,
            generator,
        ),
        Mode::Monitor => monitor::spawn_network(
            session_settings,
            network_spawn_params,
            quick_setup,
            generator,
        ),
        Mode::Interactive => interactive::spawn_network(
            session_settings,
            network_spawn_params,
            quick_setup,
            generator,
        ),
        Mode::Service => service::spawn_network(
            session_settings,
            network_spawn_params,
            quick_setup,
            generator,
        ),
    }
}

pub struct NetworkSpawnParams {
    token: Option<String>,
    endpoint: String,
    protocol: Protocol,
    version: String,
    working_directory: PathBuf,
}

impl NetworkSpawnParams {
    pub fn new<P: AsRef<Path>>(
        endpoint: String,
        parameters: &VitStartParameters,
        token: Option<String>,
        working_directory: P,
    ) -> Self {
        Self {
            token,
            endpoint: endpoint.clone(),
            protocol: parameters.protocol.clone(),
            version: parameters.version.clone(),
            working_directory: working_directory.as_ref().to_path_buf(),
        }
    }

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn nodes_params(&self) -> Vec<SpawnParams> {
        vec![
            SpawnParams::new(LEADER_1)
                .leader()
                .persistence_mode(PersistenceMode::Persistent),
            SpawnParams::new(LEADER_2)
                .leader()
                .persistence_mode(PersistenceMode::Persistent),
            SpawnParams::new(LEADER_3)
                .leader()
                .persistence_mode(PersistenceMode::Persistent),
            SpawnParams::new(WALLET_NODE)
                .passive()
                .persistence_mode(PersistenceMode::Persistent)
                .persistent_fragment_log(self.working_directory.clone().join("persistent_log")),
        ]
    }

    pub fn proxy_params(&self) -> WalletProxySpawnParams {
        *WalletProxySpawnParams::new(WALLET_NODE)
            .with_base_address(self.endpoint.clone())
            .with_protocol(self.protocol.clone())
    }
}
