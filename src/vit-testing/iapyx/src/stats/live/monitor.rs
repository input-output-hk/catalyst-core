use crate::stats::live::Harvester;
use crate::stats::live::Settings;
use jortestkit::load::ProgressBar;
use jortestkit::console::ProgressBarMode;
use jortestkit::prelude::append;
use std::{
    sync::mpsc::{self, Sender, TryRecvError},
    thread::{self, JoinHandle},
};

pub struct MonitorThread {
    stop_signal: Sender<()>,
    handle: JoinHandle<()>,
}

impl MonitorThread {
    pub fn start(harvester: Harvester, settings: Settings, title: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut progress_bar = ProgressBar::new(1);

        println!("{}", title);
        jortestkit::load::use_as_monitor_progress_bar(
            &settings.monitor(),
            title,
            &mut progress_bar,
        );

        let monitor = thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    progress_bar.finish_and_clear();
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
            let stats = harvester.harvest().unwrap();

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
                if logger.exists() {
                    append(logger, stats.entry()).unwrap();
                } else {
                    std::fs::File::create(logger).unwrap();
                    append(logger, stats.header()).unwrap();
                    append(logger, stats.entry()).unwrap();
                }
            }
            thread::sleep(std::time::Duration::from_secs(settings.interval));
        });

        Self {
            stop_signal: tx,
            handle: monitor,
        }
    }

    pub fn stop(self) {
        self.stop_signal.send(()).unwrap();
        self.handle.join().unwrap();
    }
}
