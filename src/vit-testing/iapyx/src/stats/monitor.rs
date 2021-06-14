use indicatif::ProgressBar;
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
    pub fn start(harvester: Harvester, settings: Settings, title: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let request_clone = Arc::clone(&requests);
        let mut progress_bar = ProgressBar::new(1);

        println!("{}",title);
        use_as_monitor_progress_bar(&monitor, stats.header(), &mut progress_bar);

        let monitor = thread::spawn(move || {
            let timer = Instant::now();
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        progress_bar.finish_and_clear();
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                }    
                let stats = harvester.harvest();

                match monitor {
                    Monitor::Standard(_interval) => {
                        println!("{}",stats.as_console_output());
                    },
                    Monitor::Progress(_interval) => {
                        progress_bar.set_message(&progress);
                    },
                    _ => (),
                }

                if settings.logger().exists() {
                    stats.entry();
                } else {
                    stats.header();
                    stats.entry();
                }
                thread::sleep(std::time::Duration::from_millis(interval));
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
