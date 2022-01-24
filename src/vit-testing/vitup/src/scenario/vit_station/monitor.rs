use hersir::controller::ProgressBarController;
use hersir::style;
use jormungandr_automation::jormungandr::{NodeAlias, Status};
use std::io::BufReader;
pub type VitStationSettings = vit_servicing_station_lib::server::settings::ServiceSettings;
use super::Result;
use crate::vit_station::VitStationController;
use std::io::BufRead;

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

    pub fn finish_monitoring(&self) {
        self.progress_bar.finish_with_message("monitoring shutdown");
    }

    pub fn progress_bar(&self) -> &ProgressBarController {
        &self.progress_bar
    }

    pub fn shutdown(&self) {
        self.controller.shutdown();
    }

    pub fn capture_logs(&mut self) {
        let stderr = self.controller.std_err().take().unwrap();
        let reader = BufReader::new(stderr);
        for line_result in reader.lines() {
            let line = line_result.expect("failed to read a line from log output");
            self.progress_bar.log_info(&line);
        }
    }

    fn progress_bar_start(&self) {
        self.progress_bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}")
                .tick_chars(style::TICKER),
        );
        self.progress_bar.enable_steady_tick(100);
        self.progress_bar.set_message(&format!(
            "{} {} ... [{}] Vit station is up",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            self.controller.address(),
        ));
    }

    fn progress_bar_failure(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::error.apply_to(*style::icons::failure)
        ));
    }

    fn progress_bar_success(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::success.apply_to(*style::icons::success)
        ));
    }
}
