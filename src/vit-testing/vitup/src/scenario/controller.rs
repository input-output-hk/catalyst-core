use crate::scenario::{
    settings::VitSettings,
    vit_station::{VitStation, VitStationController, VitStationControllerError},
    wallet::{
        Error as WalletProxyError, WalletProxy, WalletProxyController, WalletProxySpawnParams,
    },
};
use crate::Result;
use hersir::{
    builder::{Blockchain, Topology},
    controller::{Context, MonitorController, MonitorControllerBuilder},
};
use indicatif::ProgressBar;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;

pub struct VitControllerBuilder {
    controller_builder: MonitorControllerBuilder,
}

pub struct VitController {
    vit_settings: VitSettings,
}

impl VitControllerBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            controller_builder: MonitorControllerBuilder::new(title),
        }
    }

    pub fn topology(mut self, topology: Topology) -> Self {
        self.controller_builder = self.controller_builder.topology(topology);
        self
    }

    pub fn blockchain(mut self, blockchain: Blockchain) -> Self {
        self.controller_builder = self.controller_builder.blockchain(blockchain);
        self
    }

    pub fn build(self, mut context: Context) -> Result<(VitController, MonitorController)> {
        let vit_controller = VitController::new(VitSettings::new(&mut context));
        let controller = self.controller_builder.build(context)?;
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

    pub fn spawn_vit_station(
        &self,
        controller: &mut MonitorController,
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
        controller: &mut MonitorController,
        params: &mut WalletProxySpawnParams,
    ) -> Result<WalletProxyController> {
        let node_alias = params.alias.clone();

        let (alias, settings) = self
            .vit_settings()
            .wallet_proxies
            .iter()
            .next()
            .ok_or(WalletProxyError::NoWalletProxiesDefinedInSettings)?;
        let node_setting = if let Some(node_setting) = controller.settings().nodes.get(&node_alias)
        {
            node_setting.clone()
        } else {
            return Err(crate::error::Error::ProxyNotFound {
                alias: node_alias.to_string(),
            });
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
        controller: &mut MonitorController,
        alias: &str,
    ) -> Result<WalletProxyController> {
        self.spawn_wallet_proxy_custom(controller, &mut WalletProxySpawnParams::new(alias))
    }
}
