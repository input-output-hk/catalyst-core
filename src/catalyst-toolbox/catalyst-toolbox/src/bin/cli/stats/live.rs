use catalyst_toolbox::stats::live::{start, Harvester, Settings};
use jortestkit::console::ProgressBarMode;
use jortestkit::prelude::parse_progress_bar_mode_from_str;
use std::path::PathBuf;
use structopt::StructOpt;

/// Commands connect to desired backend and query endpoint with some interval.
/// It can dump result to to console in a progress mode/standard printout or file.
#[derive(StructOpt, Debug)]
pub struct LiveStatsCommand {
    #[structopt(long = "endpoint")]
    pub endpoint: String,
    #[structopt(long = "interval")]
    pub interval: u64,
    #[structopt(
        long = "progress-bar-mode",
        default_value = "Monitor",
        parse(from_str = parse_progress_bar_mode_from_str)
    )]
    pub console: ProgressBarMode,
    #[structopt(long = "logger")]
    pub file: Option<PathBuf>,
    #[structopt(long = "duration")]
    pub duration: u64,
}

impl LiveStatsCommand {
    pub fn exec(&self) -> Result<(), catalyst_toolbox::stats::Error> {
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
