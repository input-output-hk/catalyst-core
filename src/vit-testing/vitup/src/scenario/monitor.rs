use super::controller::VitController as InnerController;
use crate::vit_station::{
    ValidVotePlanParameters, ValidVotingTemplateGenerator, VitStationMonitorController,
};
use crate::wallet::WalletProxyMonitorController;
use crate::wallet::WalletProxySpawnParams;
use crate::Result;
use hersir::builder::SpawnParams;
use hersir::controller::MonitorNode;
use hersir::controller::ProgressBarController;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use std::net::SocketAddr;
use std::sync::Arc;
pub struct MonitorController {
    inner: InnerController,
    hersir_controller: hersir::controller::MonitorController,
    progress_bar: Arc<MultiProgress>,
    progress_bar_thread: Option<std::thread::JoinHandle<()>>,
}

impl MonitorController {
    pub fn new(
        inner: InnerController,
        hersir_controller: hersir::controller::MonitorController,
    ) -> Self {
        let progress_bar = Arc::new(MultiProgress::new());

        Self {
            inner,
            hersir_controller,
            progress_bar,
            progress_bar_thread: None,
        }
    }

    pub fn monitor_nodes(&mut self) {
        let pb = Arc::clone(&self.progress_bar);
        self.progress_bar_thread = Some(std::thread::spawn(move || {
            pb.join().unwrap();
        }));
    }

    pub fn finalize(self) {
        if let Some(thread) = self.progress_bar_thread {
            thread.join().unwrap()
        }
    }

    fn build_progress_bar(&mut self, alias: &str, listen: SocketAddr) -> ProgressBarController {
        let pb = ProgressBar::new_spinner();
        let pb = self.add_to_progress_bar(pb);
        ProgressBarController::new(pb, format!("{}@{}", alias, listen))
    }

    pub fn add_to_progress_bar(&mut self, pb: ProgressBar) -> ProgressBar {
        self.progress_bar.add(pb)
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
        let progress_bar = self.build_progress_bar(vit_station.alias(), vit_station.address());

        Ok(VitStationMonitorController::new(vit_station, progress_bar))
    }

    pub fn spawn_wallet_proxy_custom(
        &mut self,
        params: &mut WalletProxySpawnParams,
    ) -> Result<WalletProxyMonitorController> {
        let wallet_proxy = self.inner.spawn_wallet_proxy_custom(params)?;
        let progress_bar = self.build_progress_bar(
            wallet_proxy.alias(),
            wallet_proxy.address().parse().unwrap(),
        );
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
