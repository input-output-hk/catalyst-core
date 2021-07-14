use super::request::Response;
use std::sync::{
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc, RwLock,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct ResponseCollector {
    responses: Arc<RwLock<Vec<Response>>>,
    stop_signal: Sender<()>,
    handle: JoinHandle<()>,
}

impl ResponseCollector {
    const IDLE_TIME: Duration = Duration::from_millis(100);
    const TIMEOUT: Duration = Duration::from_millis(50);

    pub fn start(resp_rx: Receiver<Vec<Response>>) -> Self {
        let (tx, rx) = mpsc::channel::<()>();
        let responses = Arc::new(RwLock::new(Vec::new()));
        let responses_clone = responses.clone();

        let handle = std::thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    let mut req = responses_clone.write().unwrap();
                    while let Ok(resp) = resp_rx.try_recv() {
                        req.extend(resp);
                    }
                    break;
                }
                Err(TryRecvError::Empty) => {
                    let mut req = Vec::new();
                    let start = Instant::now();
                    while let Ok(resp) = resp_rx.try_recv() {
                        req.extend(resp);
                        if start.elapsed() > Self::TIMEOUT {
                            break; // do not hold the lock for too long
                        }
                    }
                    responses_clone.write().unwrap().extend(req);
                }
            }
            thread::sleep(Self::IDLE_TIME);
        });
        Self {
            stop_signal: tx,
            handle,
            responses,
        }
    }

    pub fn stop(self) -> Arc<RwLock<Vec<Response>>> {
        self.stop_signal.send(()).unwrap();
        self.handle.join().unwrap();
        self.responses
    }

    pub fn responses(&self) -> &Arc<RwLock<Vec<Response>>> {
        &self.responses
    }
}
