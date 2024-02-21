use super::{
    super::VitSettings,
    vit_station::{
        controller::VitStationController, controller::VIT_CONFIG, dump_settings_to_file,
        BootstrapCommandBuilder, DbGenerator, Error as VitStationControllerError, RestClient,
        ValidVotePlanParameters, ValidVotingTemplateGenerator,
    },
    wallet_proxy::{Error as WalletProxyError, WalletProxyController, WalletProxySpawnParams},
};
use crate::builders::ArchiveConfiguration;
use crate::mode::standard::settings::{VIT_STATION, VIT_STATION_ARCHIVE};
use crate::mode::standard::ExplorerController;
use crate::Result;
use assert_fs::fixture::PathChild;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use hersir::builder::ControllerError;
use hersir::config::{
    BlockchainConfiguration, CommitteeTemplate, ExplorerTemplate, SpawnParams, VotePlanTemplate,
    WalletTemplate,
};
use hersir::{
    builder::{
        NetworkBuilder, NodeAlias, NodeSetting, Settings, Topology, Wallet as WalletSettings,
    },
    config::SessionSettings,
};
use jormungandr_automation::jormungandr::{JormungandrProcess, Status, TestingDirectory};
use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use thor::{Wallet, WalletAlias};

#[derive(Default)]
pub struct VitControllerBuilder {
    committees: Vec<CommitteeTemplate>,
    controller_builder: NetworkBuilder,
    archive_conf: Option<ArchiveConfiguration>,
}

impl VitControllerBuilder {
    pub fn new() -> Self {
        Self {
            committees: Vec::new(),
            controller_builder: NetworkBuilder::default(),
            archive_conf: None,
        }
    }

    pub(crate) fn archive_conf(mut self, archive_conf: ArchiveConfiguration) -> Self {
        self.archive_conf = Some(archive_conf);
        self
    }

    pub(crate) fn committee(mut self, committee: CommitteeTemplate) -> Self {
        self.committees.push(committee);
        self
    }

    pub fn topology(mut self, topology: Topology) -> Self {
        self.controller_builder = self.controller_builder.topology(topology);
        self
    }

    pub fn blockchain(mut self, blockchain: BlockchainConfiguration) -> Self {
        self.controller_builder = self.controller_builder.blockchain_config(blockchain);
        self
    }

    pub fn wallets(mut self, wallet_templates: Vec<WalletTemplate>) -> Self {
        self.controller_builder = self.controller_builder.wallet_templates(wallet_templates);
        self
    }

    pub fn wallet(mut self, wallet_template: WalletTemplate) -> Self {
        self.controller_builder = self.controller_builder.wallet_template(wallet_template);
        self
    }

    pub fn vote_plans(mut self, vote_plans: Vec<VotePlanTemplate>) -> Self {
        self.controller_builder = self.controller_builder.vote_plan_templates(vote_plans);
        self
    }
    pub fn explorer(mut self, explorer: ExplorerTemplate) -> Self {
        self.controller_builder = self.controller_builder.explorer(Some(explorer));
        self
    }

    pub fn build(
        self,
        mut session_settings: SessionSettings,
    ) -> std::result::Result<VitController, Error> {
        let controller = self
            .controller_builder
            .committees(self.committees)
            .session_settings(session_settings.clone())
            .build()?;

        Ok(VitController::new(
            VitSettings::new(&mut session_settings, self.archive_conf),
            controller,
        ))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Controller(#[from] ControllerError),
    #[error("cannot bootstrap vit station server. health checkpoint is rejecting request")]
    CannotBootstrap,
    #[error("cannot bootstrap explorer. health checkpoint is rejecting request")]
    CannotBootstrapExplorer,
    #[error("cannot get wallet with alias {alias}, either does not exist or controller does not have any control over it")]
    CannotGetWallet { alias: String },
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
        self.hersir_controller
            .controlled_wallet(wallet)
            .ok_or(Error::CannotGetWallet {
                alias: wallet.to_owned(),
            })
            .map_err(Into::into)
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

    pub fn controlled_wallets(&self) -> Vec<Wallet> {
        self.hersir_controller.controlled_wallets()
    }

    pub fn defined_wallets(&self) -> Vec<(WalletAlias, &WalletSettings)> {
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

    pub fn spawn_vit_station_archive(&self, version: String) -> Result<VitStationController> {
        let settings = self
            .vit_settings
            .vit_stations
            .get(VIT_STATION_ARCHIVE)
            .ok_or(VitStationControllerError::NoVitStationArchiverDefinedInSettings)?;

        let mut command_builder =
            BootstrapCommandBuilder::new(PathBuf::from("vit-servicing-station-server"));

        let mut command = command_builder
            .db_url(&settings.db_url)
            .block0_paths(
                settings
                    .block0_paths
                    .clone()
                    .map(|p| p.to_str().unwrap().to_string()),
            )
            .service_version(version)
            .build();

        println!("Starting vit-servicing-station-archiver: {:?}", command);

        let controller = VitStationController {
            alias: VIT_STATION_ARCHIVE.to_string(),
            rest_client: RestClient::from(settings),
            process: command.spawn().unwrap(),
            settings: settings.clone(),
            status: Arc::new(Mutex::new(Status::Running)),
        };

        wait_for_bootstrap(&controller)?;

        Ok(controller)
    }

    //TODO: move to vit station builder
    pub fn spawn_vit_station(
        &self,
        vote_plan_parameters: ValidVotePlanParameters,
        template_generator: &mut dyn ValidVotingTemplateGenerator,
        version: String,
    ) -> Result<VitStationController> {
        let settings = self
            .vit_settings
            .vit_stations
            .get(VIT_STATION)
            .ok_or(VitStationControllerError::NoVitStationDefinedInSettings)?;

        let working_directory = self.hersir_controller.working_directory().path();

        let dir = working_directory.join(VIT_STATION);
        std::fs::DirBuilder::new().recursive(true).create(&dir)?;

        let config_file = dir.join(VIT_CONFIG);
        dump_settings_to_file(config_file.to_str().unwrap(), settings).unwrap();

        let db_url = DbGenerator::new(vote_plan_parameters).build(template_generator)?;

        let mut command_builder =
            BootstrapCommandBuilder::new(PathBuf::from("vit-servicing-station-server"));
        let mut command = command_builder
            .in_settings_file(&config_file)
            .db_url(db_url)
            .service_version(version)
            .block0_path(Some(
                self.hersir_controller
                    .block0_file()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ))
            .build();

        println!("Starting vit-servicing-station: {:?}", command);

        let controller = VitStationController {
            alias: VIT_STATION.to_string(),
            rest_client: RestClient::from(settings),
            process: command.spawn().unwrap(),
            settings: settings.clone(),
            status: Arc::new(Mutex::new(Status::Running)),
        };

        wait_for_bootstrap(&controller)?;

        Ok(controller)
    }

    pub fn spawn_explorer(&mut self) -> Result<ExplorerController> {
        let explorer = self.hersir_controller.spawn_explorer()?;
        if !explorer.wait_to_be_up(1, 3) {
            Err(Error::CannotBootstrapExplorer)?
        } else {
            Ok(ExplorerController::new("explorer".to_string(), explorer))
        }
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
        std::fs::DirBuilder::new().recursive(true).create(dir)?;

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

        println!("proxy: {:?}", command);

        if let valgrind::Protocol::Https(certs) = &params.protocol {
            command
                .arg("--cert")
                .arg(&certs.cert_path)
                .arg("--key")
                .arg(&certs.key_path);
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

fn wait_for_bootstrap(controller: &VitStationController) -> std::result::Result<(), Error> {
    std::thread::sleep(std::time::Duration::from_secs(5));

    for _ in 0..5 {
        if controller.check_running() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Err(Error::CannotBootstrap)
}
