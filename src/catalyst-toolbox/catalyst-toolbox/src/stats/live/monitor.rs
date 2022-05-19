use std::time::Instant;

use super::Error;
use crate::stats::live::Harvester;
use crate::stats::live::Settings;
use jortestkit::console::ProgressBarMode;
use jortestkit::load::ProgressBar;
use jortestkit::prelude::append;

pub fn start(harvester: Harvester, settings: Settings, title: &str) -> Result<(), Error> {
    let mut progress_bar = ProgressBar::new(1);

    println!("{}", title);
    jortestkit::load::use_as_monitor_progress_bar(&settings.monitor(), title, &mut progress_bar);
    let start = Instant::now();

    loop {
        if settings.duration > start.elapsed().as_secs() {
            break;
        }

        let stats = harvester.harvest()?;

        match settings.progress_bar_mode() {
            ProgressBarMode::Standard => {
                println!("{}", stats.to_console_output());
            }
            ProgressBarMode::Monitor => {
                progress_bar.set_message(&stats.to_console_output());
            }
            _ => (),
        }

        if let Some(logger) = &settings.logger {
            if !logger.exists() {
                std::fs::File::create(logger)?;
            }
            append(logger, stats.header())?;
        }
        std::thread::sleep(std::time::Duration::from_secs(settings.interval));
    }

    progress_bar.finish_and_clear();
    Ok(())
}
