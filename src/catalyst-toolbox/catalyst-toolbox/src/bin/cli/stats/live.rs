use catalyst_toolbox::stats::live::{start, Harvester, Settings};
use color_eyre::Report;
use jortestkit::console::ProgressBarMode;
use jortestkit::prelude::parse_progress_bar_mode_from_str;
use std::path::PathBuf;
use clap::Parser;

/// Commands connect to desired backend and query endpoint with some interval.
/// It can dump result to to console in a progress mode/standard printout or file.
#[derive(Parser, Debug)]
pub struct LiveStatsCommand {
    #[clap(long = "endpoint")]
    pub endpoint: String,
    #[clap(long = "interval")]
    pub interval: u64,
    #[clap(
        long = "progress-bar-mode",
        default_value = "Monitor",
        value_parser = parse_progress_bar_mode_from_str
    )]
    pub console: ProgressBarMode,
    #[clap(long = "logger")]
    pub file: Option<PathBuf>,
    #[clap(long = "duration")]
    pub duration: u64,
}

impl LiveStatsCommand {
    pub fn exec(&self) -> Result<(), Report> {
        let settings = Settings {
            endpoint: self.endpoint.clone(),
            progress: self.console,
            interval: self.interval,
            logger: self.file.clone(),
            duration: self.duration,
        };

        let harvester = Harvester::new(
            self.endpoint.clone(),
            std::time::Duration::from_secs(self.interval),
        );

        start(
            harvester,
            settings,
            &format!("{} monitoring", self.endpoint),
        )
        .map_err(Into::into)
    }
}
