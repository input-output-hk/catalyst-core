use jortestkit::console::ProgressBarMode;
use jortestkit::load::Monitor;
use std::path::PathBuf;

pub struct Settings {
    pub endpoint: String,
    pub interval: u64,
    pub logger: Option<PathBuf>,
    pub progress: ProgressBarMode,
    pub duration: u64,
}

impl Settings {
    pub fn monitor(&self) -> Monitor {
        match self.progress {
            ProgressBarMode::Monitor => Monitor::Progress(self.interval),
            ProgressBarMode::Standard => Monitor::Standard(self.interval),
            ProgressBarMode::None => Monitor::Disabled(self.interval),
        }
    }

    pub fn progress_bar_mode(&self) -> ProgressBarMode {
        self.progress
    }
}
