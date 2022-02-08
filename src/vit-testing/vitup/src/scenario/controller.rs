use crate::scenario::{
    settings::VitSettings,
    vit_station::{Error as VitStationControllerError, VitStationController},
    wallet::{Error as WalletProxyError, WalletProxyController, WalletProxySpawnParams},
};
use crate::vit_station::BootstrapCommandBuilder;
use crate::vit_station::DbGenerator;
use crate::vit_station::RestClient;
use crate::vit_station::STORAGE;
use crate::vit_station::VIT_CONFIG;
use crate::Result;
use assert_fs::fixture::PathChild;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use hersir::builder::NodeAlias;
use hersir::builder::NodeSetting;
use hersir::builder::Settings;
use hersir::builder::SpawnParams;
use hersir::builder::Wallet as WalletSettings;
use hersir::builder::{Blockchain, NetworkBuilder, Topology};
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::JormungandrProcess;
use jormungandr_automation::jormungandr::Status;
use jormungandr_automation::jormungandr::TestingDirectory;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use thor::Wallet;
use thor::WalletAlias;
use valgrind::Protocol;
use vit_servicing_station_lib::server::settings::dump_settings_to_file;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;
#[derive(Default)]
pub struct VitControllerBuilder {
    controller_builder: NetworkBuilder,
}

impl VitControllerBuilder {
    pub fn new() -> Self {
        Self {
            controller_builder: NetworkBuilder::default(),
        }
    }

    pub fn topology(mut self, topology: Topology) -> Self {
        self.controller_builder = self.controller_builder.topology(topology);
        self
    }

    pub fn blockchain(mut self, blockchain: Blockchain) -> Self {
        self.controller_builder = self.controller_builder.blockchain_config(blockchain);
        self
    }

    pub fn build(self, mut session_settings: SessionSettings) -> Result<VitController> {
        let controller = self
            .controller_builder
            .session_settings(session_settings.clone())
            .build()?;
        Ok(VitController::new(
            VitSettings::new(&mut session_settings),
            controller,
        ))
    }
}

#[derive(Clone)]
pub struct VitController {
    vit_settings: VitSettings,
    hersir_controller: hersir::controller::Controller,
}

impl VitController {
    pub fn new(
        vit_settings: VitSettings,
        hersir_controller: hersir::controller::Controller,
    ) -> Self {
        Self {
            vit_settings,
            hersir_controller,
        }
    }

    pub fn vit_settings(&self) -> &VitSettings {
        &self.vit_settings
    }

    pub fn hersir_controller(&self) -> hersir::controller::Controller {
        self.hersir_controller.clone()
    }

    pub fn wallet(&mut self, wallet: &str) -> Result<Wallet> {
        self.hersir_controller.wallet(wallet).map_err(Into::into)
    }

    pub fn spawn_node(&mut self, spawn_params: SpawnParams) -> Result<JormungandrProcess> {
        self.hersir_controller
            .spawn(spawn_params)
            .map_err(Into::into)
    }

    pub fn settings(&self) -> Settings {
        self.hersir_controller.settings().clone()
    }

    pub fn block0_file(&self) -> PathBuf {
        self.hersir_controller.block0_file()
    }

    pub fn defined_nodes(&self) -> Vec<(&NodeAlias, &NodeSetting)> {
        self.hersir_controller.defined_nodes().collect()
    }

    pub fn defined_wallets(&self) -> Vec<(&WalletAlias, &WalletSettings)> {
        self.hersir_controller.defined_wallets().collect()
    }

    pub fn defined_vote_plan(&self, alias: &str) -> Result<VotePlanDef> {
        self.hersir_controller
            .defined_vote_plan(alias)
            .map_err(Into::into)
    }

    pub fn defined_vote_plans(&self) -> Vec<VotePlanDef> {
        self.hersir_controller.defined_vote_plans()
    }

    pub fn working_directory(&self) -> &TestingDirectory {
        self.hersir_controller.working_directory()
    }

    //TODO: move to vit station builder
    pub fn spawn_vit_station(
        &self,
        vote_plan_parameters: ValidVotePlanParameters,
        template_generator: &mut dyn ValidVotingTemplateGenerator,
        version: String,
    ) -> Result<VitStationController> {
        let (alias, settings) = self
            .vit_settings
            .vit_stations
            .iter()
            .next()
            .ok_or(VitStationControllerError::NoVitStationDefinedInSettings)?;

        let working_directory = self.hersir_controller.working_directory().path();

        let dir = working_directory.join(alias);
        std::fs::DirBuilder::new().recursive(true).create(&dir)?;

        let config_file = dir.join(VIT_CONFIG);
        let db_file = dir.join(STORAGE);
        dump_settings_to_file(config_file.to_str().unwrap(), settings).unwrap();

        DbGenerator::new(vote_plan_parameters, None).build(&db_file, template_generator);

        let mut command_builder =
            BootstrapCommandBuilder::new(PathBuf::from("vit-servicing-station-server"));
        let mut command = command_builder
            .in_settings_file(&config_file)
            .db_url(db_file.to_str().unwrap())
            .service_version(version)
            .block0_path(self.hersir_controller.block0_file().to_str().unwrap())
            .build();

        Ok(VitStationController {
            alias: alias.into(),
            rest_client: RestClient::new(settings.address.to_string()),
            process: command.spawn().unwrap(),
            settings: settings.clone(),
            status: Arc::new(Mutex::new(Status::Running)),
        })
    }

    //TODO: move to wallet builder
    pub fn spawn_wallet_proxy_custom(
        &self,
        params: &mut WalletProxySpawnParams,
    ) -> Result<WalletProxyController> {
        let node_alias = params.alias.clone();

        let (alias, settings) = self
            .vit_settings()
            .wallet_proxies
            .iter()
            .next()
            .ok_or(WalletProxyError::NoWalletProxiesDefinedInSettings)?;
        let node_setting =
            if let Some(node_setting) = self.hersir_controller.settings().nodes.get(&node_alias) {
                node_setting.clone()
            } else {
                return Err(crate::error::Error::ProxyNotFound {
                    alias: node_alias.to_string(),
                });
            };

        let mut settings_overriden = settings.clone();
        params.override_settings(&mut settings_overriden);

        let block0_file = self.hersir_controller.block0_file();
        let working_directory = self.hersir_controller.working_directory();

        let dir = working_directory.child(alias);
        std::fs::DirBuilder::new().recursive(true).create(&dir)?;

        settings_overriden.node_backend_address = Some(node_setting.config.rest.listen);

        let mut command = Command::new("valgrind");
        command
            .arg("--address")
            .arg(settings_overriden.base_address().to_string())
            .arg("--vit-address")
            .arg(&settings_overriden.base_vit_address().to_string())
            .arg("--node-address")
            .arg(
                &settings_overriden
                    .base_node_backend_address()
                    .unwrap()
                    .to_string(),
            )
            .arg("--block0")
            .arg(block0_file.as_path().to_str().unwrap());

        if let Protocol::Https {
            key_path,
            cert_path,
        } = params.protocol.clone()
        {
            command
                .arg("--cert")
                .arg(cert_path)
                .arg("--key")
                .arg(key_path);
        }

        WalletProxyController::new(
            alias.into(),
            settings.clone(),
            Arc::new(Mutex::new(Status::Running)),
            command.spawn().unwrap(),
        )
        .map_err(Into::into)
    }

    //TODO: move to wallet builder
    pub fn spawn_wallet_proxy(&self, alias: &str) -> Result<WalletProxyController> {
        self.spawn_wallet_proxy_custom(&mut WalletProxySpawnParams::new(alias))
    }
}
