use super::{VitStationMonitorController, WalletProxyMonitorController};
use crate::mode::monitor::ExplorerMonitorController;
use crate::mode::standard::{
    ValidVotePlanParameters, ValidVotingTemplateGenerator, VitController as InnerController,
    WalletProxySpawnParams,
};
use crate::Result;
use hersir::config::SpawnParams;
use hersir::controller::MonitorNode;

pub struct MonitorController {
    inner: InnerController,
    hersir_controller: hersir::controller::MonitorController,
}

impl MonitorController {
    pub fn new(
        inner: InnerController,
        hersir_controller: hersir::controller::MonitorController,
    ) -> Self {
        Self {
            inner,
            hersir_controller,
        }
    }

    pub fn monitor_nodes(&mut self) {
        self.hersir_controller.monitor_nodes()
    }

    pub fn finalize(self) {
        self.hersir_controller.finalize()
    }

    pub fn spawn_vit_station(
        &mut self,
        vote_plan_parameters: ValidVotePlanParameters,
        template_generator: &mut dyn ValidVotingTemplateGenerator,
        version: String,
    ) -> Result<VitStationMonitorController> {
        let vit_station =
            self.inner
                .spawn_vit_station(vote_plan_parameters, template_generator, version)?;
        let progress_bar = self
            .hersir_controller
            .build_progress_bar(vit_station.alias(), vit_station.address());

        Ok(VitStationMonitorController::new(vit_station, progress_bar))
    }

    pub fn spawn_vit_station_archiver(
        &mut self,
        version: String,
    ) -> Result<VitStationMonitorController> {
        let vit_station = self.inner.spawn_vit_station_archive(version)?;
        let progress_bar =
            self.build_progress_bar(vit_station.alias(), vit_station.address().to_string());

        Ok(VitStationMonitorController::new(vit_station, progress_bar))
    }

    pub fn spawn_explorer(&mut self) -> Result<ExplorerMonitorController> {
        let explorer = self.inner.spawn_explorer()?;
        let progress_bar = self.build_progress_bar(explorer.alias(), explorer.address());

        Ok(ExplorerMonitorController::new(explorer, progress_bar))
    }

    pub fn spawn_wallet_proxy_custom(
        &mut self,
        params: &mut WalletProxySpawnParams,
    ) -> Result<WalletProxyMonitorController> {
        let wallet_proxy = self.inner.spawn_wallet_proxy_custom(params)?;
        let progress_bar = self
            .hersir_controller
            .build_progress_bar(wallet_proxy.alias(), wallet_proxy.settings().base_address());
        Ok(WalletProxyMonitorController::new(
            wallet_proxy,
            progress_bar,
        ))
    }

    pub fn spawn_wallet_proxy(&mut self, alias: &str) -> Result<WalletProxyMonitorController> {
        self.spawn_wallet_proxy_custom(&mut WalletProxySpawnParams::new(alias))
    }

    pub fn spawn_node(&mut self, spawn_params: SpawnParams) -> Result<MonitorNode> {
        self.hersir_controller
            .spawn_node_custom(spawn_params)
            .map_err(Into::into)
    }
}
