use crate::mode::standard::VitStationController;
use crate::Result;
use hersir::controller::ProgressBarController;
use hersir::style;
use jormungandr_automation::jormungandr::{NodeAlias, Status};
use vit_servicing_station_tests::common::clients::RestClient;

pub struct VitStationMonitorController {
    controller: VitStationController,
    progress_bar: ProgressBarController,
}

impl VitStationMonitorController {
    pub fn new(controller: VitStationController, progress_bar: ProgressBarController) -> Self {
        let monitor = Self {
            controller,
            progress_bar,
        };
        monitor.progress_bar_start();
        monitor
    }

    pub fn alias(&self) -> NodeAlias {
        self.controller.alias().to_string()
    }

    pub fn status(&self) -> Result<Status> {
        //TODO: add proper erro handling
        Ok(self.controller.status())
    }

    pub fn is_up(&self) -> bool {
        match self.status() {
            Ok(status) => status == Status::Running,
            Err(_) => false,
        }
    }

    pub fn rest(&self) -> RestClient {
        self.controller.rest_client.clone()
    }

    pub fn finish_monitoring(&self) {
        self.progress_bar.finish_with_message("monitoring shutdown");
    }

    pub fn progress_bar(&self) -> &ProgressBarController {
        &self.progress_bar
    }

    pub fn shutdown(mut self) {
        self.controller.shutdown();
    }

    fn progress_bar_start(&self) {
        self.progress_bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}")
                .tick_chars(style::TICKER),
        );
        self.progress_bar.enable_steady_tick(1000);
        self.progress_bar.log_info("vit servicing station is up");
    }
    #[allow(dead_code)]
    fn progress_bar_failure(&self) {
        self.progress_bar.finish_with_message(format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::error.apply_to(*style::icons::failure)
        ));
    }

    #[allow(dead_code)]
    fn progress_bar_success(&self) {
        self.progress_bar.finish_with_message(format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::success.apply_to(*style::icons::success)
        ));
    }
}
