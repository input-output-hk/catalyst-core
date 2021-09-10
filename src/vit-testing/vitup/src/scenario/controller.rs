use crate::scenario::{
    settings::VitSettings,
    vit_station::{VitStation, VitStationController, VitStationControllerError},
    wallet::{
        Error as WalletProxyError, WalletProxy, WalletProxyController, WalletProxySpawnParams,
    },
};
use crate::{error::ErrorKind, Result};
use iapyx::WalletBackend;
use indicatif::ProgressBar;
use jormungandr_scenario_tests::scenario::{ContextChaCha, Controller, ControllerBuilder};
use jormungandr_testing_utils::testing::network_builder::{Blockchain, Topology};
use std::path::Path;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub struct VitControllerBuilder {
    controller_builder: ControllerBuilder,
    vit_settings: Option<VitSettings>,
}

pub struct VitController {
    vit_settings: VitSettings,
}

impl VitControllerBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            controller_builder: ControllerBuilder::new(title),
            vit_settings: None,
        }
    }

    pub fn set_topology(&mut self, topology: Topology) {
        self.controller_builder.set_topology(topology);
    }

    pub fn set_blockchain(&mut self, blockchain: Blockchain) {
        self.controller_builder.set_blockchain(blockchain);
    }

    pub fn build_settings(&mut self, context: &mut ContextChaCha) {
        self.controller_builder.build_settings(context);
        self.vit_settings = Some(VitSettings::new(context));
    }

    pub fn build_controllers(self, context: ContextChaCha) -> Result<(VitController, Controller)> {
        let controller = self.controller_builder.build(context)?;
        let vit_controller = VitController::new(self.vit_settings.unwrap());
        Ok((vit_controller, controller))
    }
}

impl VitController {
    pub fn new(vit_settings: VitSettings) -> Self {
        Self { vit_settings }
    }

    pub fn vit_settings(&self) -> &VitSettings {
        &self.vit_settings
    }

    /// iapyx wallet is a mock mobile wallet
    /// it uses some production code while handling wallet operation
    // therefore controller has separate method to build such wallet
    pub fn iapyx_wallet_from_mnemonics(
        &self,
        mnemonics: &str,
        wallet_proxy: &WalletProxyController,
    ) -> Result<iapyx::Controller> {
        let settings = iapyx::WalletBackendSettings {
            use_https: false,
            enable_debug: true,
            certificate: None,
        };

        let backend = WalletBackend::new_from_addresses(
            wallet_proxy.settings().base_address().to_string(),
            wallet_proxy.settings().base_address().to_string(),
            wallet_proxy.settings().base_address().to_string(),
            settings,
        );

        Ok(iapyx::Controller::recover_with_backend(
            backend,
            mnemonics,
            &[],
        )?)
    }

    /// iapyx wallet is a mock mobile wallet
    /// it uses some production code while handling wallet operation
    // therefore controller has separate method to build such wallet
    pub fn iapyx_wallet_from_secret<P: AsRef<Path>>(
        &self,
        secret: P,
        wallet_proxy: &WalletProxyController,
    ) -> Result<iapyx::Controller> {
        let settings = iapyx::WalletBackendSettings {
            use_https: false,
            enable_debug: true,
            certificate: None,
        };

        let backend = WalletBackend::new_from_addresses(
            wallet_proxy.settings().base_address().to_string(),
            wallet_proxy.settings().base_address().to_string(),
            wallet_proxy.settings().base_address().to_string(),
            settings,
        );

        Ok(iapyx::Controller::recover_from_sk(backend, secret)?)
    }

    /// iapyx wallet is a mock mobile wallet
    /// it uses some production code while handling wallet operation
    // therefore controller has separate method to build such wallet
    pub fn iapyx_wallet_from_qr<P: AsRef<Path>>(
        &self,
        qr: P,
        password: &str,
        wallet_proxy: &WalletProxyController,
    ) -> Result<iapyx::Controller> {
        let settings = iapyx::WalletBackendSettings {
            use_https: false,
            enable_debug: true,
            certificate: None,
        };

        Ok(iapyx::Controller::recover_from_qr(
            wallet_proxy.settings().base_address().to_string(),
            qr,
            password,
            settings,
        )
        .unwrap())
    }

    pub fn spawn_vit_station(
        &self,
        controller: &mut Controller,
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

        let pb = ProgressBar::new_spinner();
        let pb = controller.add_to_progress_bar(pb);

        let block0_file = controller.block0_file();
        let working_directory = controller.working_directory().path();

        let vit_station = VitStation::spawn(
            controller.context(),
            vote_plan_parameters,
            template_generator,
            pb,
            alias,
            settings.clone(),
            block0_file.as_path(),
            working_directory,
            &version,
        )
        .unwrap();
        Ok(vit_station.controller())
    }

    pub fn spawn_wallet_proxy_custom(
        &self,
        controller: &mut Controller,
        params: &mut WalletProxySpawnParams,
    ) -> Result<WalletProxyController> {
        let node_alias = params.alias.clone();

        let (alias, settings) = self
            .vit_settings()
            .wallet_proxies
            .iter()
            .next()
            .ok_or(WalletProxyError::NoWalletProxiesDefinedInSettings)?;
        let node_setting = if let Some(node_setting) = controller
            .settings()
            .network_settings
            .nodes
            .get(&node_alias)
        {
            node_setting.clone()
        } else {
            bail!(ErrorKind::ProxyNotFound(node_alias.to_string()))
        };

        let mut settings_overriden = settings.clone();
        params.override_settings(&mut settings_overriden);

        let pb = ProgressBar::new_spinner();
        let pb = controller.add_to_progress_bar(pb);

        let block0_file = controller.block0_file();
        let working_directory = controller.working_directory();

        let wallet_proxy = WalletProxy::spawn(
            controller.context(),
            pb,
            alias,
            settings_overriden,
            &node_setting,
            block0_file.as_path(),
            working_directory.path(),
            params.protocol.clone(),
        )
        .unwrap();
        Ok(wallet_proxy.controller())
    }

    pub fn spawn_wallet_proxy(
        &self,
        controller: &mut Controller,
        alias: &str,
    ) -> Result<WalletProxyController> {
        self.spawn_wallet_proxy_custom(controller, &mut WalletProxySpawnParams::new(alias))
    }
}
