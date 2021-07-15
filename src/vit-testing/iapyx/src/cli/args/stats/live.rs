use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::live::Harvester;
use crate::stats::live::MonitorThread;
use crate::stats::live::Settings;
use jortestkit::console::ProgressBarMode;
use jortestkit::prelude::parse_progress_bar_mode_from_str;
use std::path::PathBuf;
use structopt::StructOpt;

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
    pub fn exec(&self) -> Result<(), IapyxStatsCommandError> {
        let settings = Settings {
            endpoint: self.endpoint.clone(),
            progress: self.console,
            interval: self.interval,
            logger: self.file.clone(),
            duration: self.duration,
        };

        let harvester = Harvester::new(self.endpoint.clone());
        let monitor = MonitorThread::start(
            harvester,
            settings,
            &format!("{} monitoring", self.endpoint),
        );
        std::thread::sleep(std::time::Duration::from_secs(300));
        monitor.stop();
        Ok(())
    }
}
