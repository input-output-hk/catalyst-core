use super::{config::Monitor, request::Response, stats::Stats};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use std::{
    sync::{
        mpsc::{self, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct MonitorThread {
    stop_signal: Sender<()>,
    handle: JoinHandle<()>,
}

impl MonitorThread {
    fn initialize(monitor: &Monitor, title: &str) -> ProgressBar {
        let banner = format!("[Load Scenario: {}]", title);
        let progress_bar = ProgressBar::new(1);
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        progress_bar.set_style(spinner_style);

        match monitor {
            Monitor::Standard(_) => println!("{}", banner),
            Monitor::Progress(_) => {
                progress_bar.set_prefix(&banner);
                progress_bar.set_message(&"initializing...".to_string());
            }
            _ => (),
        }
        progress_bar
    }

    pub fn start(requests: &Arc<Mutex<Vec<Response>>>, monitor: Monitor, title: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let request_clone = Arc::clone(&requests);
        let progress_bar = Self::initialize(&monitor, title);
        let monitor = thread::spawn(move || {
            let timer = Instant::now();
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }
                match monitor {
                    Monitor::Standard(interval) => {
                        thread::sleep(std::time::Duration::from_millis(interval));
                        println!(
                            "{}",
                            Stats::new(
                                request_clone.clone().lock().unwrap().to_vec(),
                                timer.elapsed()
                            )
                            .tps_status()
                        );
                    }
                    Monitor::Disabled(interval) => {
                        thread::sleep(std::time::Duration::from_millis(interval));
                    }
                    Monitor::Progress(interval) => {
                        thread::sleep(std::time::Duration::from_millis(interval));
                        let progress = Stats::new(
                            request_clone.clone().lock().unwrap().to_vec(),
                            timer.elapsed(),
                        )
                        .tps_status();
                        progress_bar.set_message(&progress);
                    }
                }
            }
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
