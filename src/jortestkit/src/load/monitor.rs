use super::{
    config::Monitor, progress::use_as_monitor_progress_bar, request::Response, stats::Stats,
};
use indicatif::ProgressBar;
use std::time::Instant;
use std::{
    sync::{
        mpsc::{self, Sender, TryRecvError},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

pub struct MonitorThread {
    stop_signal: Sender<()>,
    handle: JoinHandle<()>,
}

impl MonitorThread {
    pub fn start_multi(
        requests: &Arc<RwLock<Vec<Response>>>,
        monitor: Monitor,
        mut progress_bar: ProgressBar,
        title: &str,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let request_clone = Arc::clone(requests);
        use_as_monitor_progress_bar(&monitor, title, &mut progress_bar);

        let monitor = thread::Builder::new()
            .name(format!("monitor-{}", title))
            .spawn(move || {
                let timer = Instant::now();
                loop {
                    match rx.try_recv() {
                        Ok(_) | Err(TryRecvError::Disconnected) => {
                            progress_bar.finish_and_clear();
                            break;
                        }
                        Err(TryRecvError::Empty) => {}
                    }
                    match monitor {
                        Monitor::Standard(interval) => {
                            thread::sleep(std::time::Duration::from_millis(interval));
                            println!(
                                "{}",
                                Stats::new(request_clone.read().unwrap().to_vec(), timer.elapsed())
                                    .tps_status()
                            );
                        }
                        Monitor::Disabled(interval) => {
                            thread::sleep(std::time::Duration::from_millis(interval));
                        }
                        Monitor::Progress(interval) => {
                            thread::sleep(std::time::Duration::from_millis(interval));
                            let progress =
                                Stats::new(request_clone.read().unwrap().to_vec(), timer.elapsed())
                                    .tps_status();
                            progress_bar.set_message(&progress);
                        }
                    }
                }
            })
            .unwrap();
        Self {
            stop_signal: tx,
            handle: monitor,
        }
    }

    pub fn start(requests: &Arc<RwLock<Vec<Response>>>, monitor: Monitor, title: &str) -> Self {
        let progress_bar = ProgressBar::new(1);
        Self::start_multi(requests, monitor, progress_bar, title)
    }

    pub fn stop(self) {
        self.stop_signal.send(()).unwrap();
        self.handle.join().unwrap();
    }
}
