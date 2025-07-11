use crate::{generators::ExplorerRequestGen, mjolnir_lib::MjolnirError};
use clap::Parser;
use jormungandr_automation::jormungandr::Explorer;
use jortestkit::{
    load::{ConfigurationBuilder, Monitor},
    prelude::{parse_progress_bar_mode_from_str, ProgressBarMode},
};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplorerLoadCommandError {
    #[error("Client Error")]
    ClientError(#[from] Box<MjolnirError>),
}

#[derive(Parser, Debug)]
pub struct ExplorerLoadCommand {
    /// Number of threads
    #[clap(short = 'c', long = "count", default_value = "3")]
    pub count: usize,
    /// Endpoint address in format:
    /// 127.0.0.1:80
    #[clap(short = 'e', long = "endpoint")]
    pub endpoint: String,

    /// Amount of delay (in milliseconds) between sync attempts
    #[clap(long = "delay", default_value = "50")]
    pub delay: u64,

    /// Load duration
    #[clap(short = 'd', long = "duration")]
    pub duration: u64,

    /// Show progress
    #[clap(
        long = "progress-bar-mode",
        short = 'b',
        default_value = "Monitor",
        value_parser = parse_progress_bar_mode_from_str,
    )]
    progress_bar_mode: ProgressBarMode,

    /// Prints post load measurements
    #[clap(short = 'm', long = "measure")]
    pub measure: bool,
}

impl ExplorerLoadCommand {
    pub fn exec(&self) -> Result<(), ExplorerLoadCommandError> {
        let mut explorer = Explorer::new(self.endpoint.clone());
        explorer.disable_logs();
        let mut request_gen = ExplorerRequestGen::new(explorer);
        request_gen.do_setup(Vec::new()).unwrap();

        let config = ConfigurationBuilder::duration(Duration::from_secs(self.duration))
            .thread_no(self.count)
            .step_delay(Duration::from_millis(self.delay))
            .monitor(self.build_monitor())
            .build();
        let stats = jortestkit::load::start_sync(request_gen, config, "Explorer load test");
        if self.measure {
            assert!((stats.calculate_passrate() as u32) > 95);
        }
        Ok(())
    }

    fn build_monitor(&self) -> Monitor {
        match self.progress_bar_mode {
            ProgressBarMode::Monitor => Monitor::Progress(100),
            ProgressBarMode::Standard => Monitor::Standard(100),
            ProgressBarMode::None => Monitor::Disabled(10),
        }
    }
}
