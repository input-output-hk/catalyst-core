use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use thread::JoinHandle;

mod config;
mod monitor;
mod progress;
mod request;
mod stats;
mod status;

use crate::load::request::run_request;
pub use config::{Configuration, Monitor, Strategy};
pub use monitor::MonitorThread;
pub use request::{Id, RequestFailure, RequestGenerator, RequestSendMode, RequestStatus, Response};
pub use stats::Stats;
pub use status::{RequestStatusProvider, Status, StatusUpdaterThread};

pub fn start_sync(
    request_generator: impl RequestGenerator + Clone + Send + Sized + 'static,
    config: Configuration,
    title: &str,
) -> Stats {
    let responses = Arc::new(Mutex::new(Vec::new()));
    let request_generator = Arc::new(Mutex::new(request_generator));
    let start = Instant::now();
    let child_threads = get_threads(
        &request_generator,
        &config,
        RequestSendMode::Sync,
        &responses,
    );
    let monitor = MonitorThread::start(&responses, config.monitor().clone(), title);

    for t in child_threads.into_iter() {
        let _child_threads = t.join();
    }
    monitor.stop();

    let lock_request = &mut responses.lock().unwrap();
    let stats = Stats::new(lock_request.to_vec(), start.elapsed());
    stats.print_summary(title);
    stats
}

pub struct BackgroundLoadProcess {
    responses: Arc<Mutex<Vec<Response>>>,
    threads: Vec<JoinHandle<()>>,
    monitor: MonitorThread,
    status_updater: StatusUpdaterThread,
    start: Instant,
}

impl BackgroundLoadProcess {
    pub fn stats(&self) -> Stats {
        let lock_request = &mut self.responses.lock().unwrap();
        Stats::new(lock_request.to_vec(), self.start.elapsed())
    }

    pub fn wait_for_finish(self) -> Stats {
        let stats = self.stats();

        for t in self.threads.into_iter() {
            let _child_threads = t.join();
        }

        self.monitor.stop();
        self.status_updater.stop();

        stats
    }
}

pub fn start_background_async(
    request_generator: impl RequestGenerator + Send + Sized + 'static,
    status_provider: impl RequestStatusProvider + Send + Sized + Sync + 'static,
    config: Configuration,
    title: &str,
) -> BackgroundLoadProcess {
    let responses = Arc::new(Mutex::new(Vec::new()));
    let request_generator = Arc::new(Mutex::new(request_generator));
    let start = Instant::now();
    let child_threads = get_threads(
        &request_generator,
        &config,
        RequestSendMode::Async,
        &responses,
    );
    let monitor = MonitorThread::start(&responses, config.monitor().clone(), title);
    let request_provider = Arc::new(Mutex::new(status_provider));
    let status_updater = StatusUpdaterThread::spawn(
        &responses,
        &request_provider,
        config.monitor().clone(),
        title,
        config.shutdown_grace_period(),
    );

    BackgroundLoadProcess {
        responses,
        threads: child_threads,
        monitor,
        status_updater,
        start,
    }
}

pub fn start_async(
    request_generator: impl RequestGenerator + Send + Sized + 'static,
    status_provider: impl RequestStatusProvider + Send + Sized + Sync + 'static,
    config: Configuration,
    title: &str,
) -> Stats {
    let responses = Arc::new(Mutex::new(Vec::new()));
    let request_generator = Arc::new(Mutex::new(request_generator));
    let start = Instant::now();
    let child_threads = get_threads(
        &request_generator,
        &config,
        RequestSendMode::Async,
        &responses,
    );
    let monitor = MonitorThread::start(&responses, config.monitor().clone(), title);
    let request_provider = Arc::new(Mutex::new(status_provider));
    let status_updater = StatusUpdaterThread::spawn(
        &responses,
        &request_provider,
        config.monitor().clone(),
        title,
        config.shutdown_grace_period(),
    );
    for t in child_threads.into_iter() {
        let _child_threads = t.join();
    }

    monitor.stop();
    status_updater.stop();

    let lock_request = &mut responses.lock().unwrap();
    let stats = Stats::new(lock_request.to_vec(), start.elapsed());
    stats.print_summary(title);
    stats
}

fn get_threads(
    request_generator: &Arc<Mutex<impl RequestGenerator + Send + Sized + 'static>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    responses: &Arc<Mutex<Vec<Response>>>,
) -> Vec<JoinHandle<()>> {
    println!("Running load using {:?}", config.strategy());
    match config.strategy() {
        Strategy::PerThread(per_thread) => per_thread_strategy(
            *per_thread,
            &responses,
            config,
            request_mode_run,
            request_generator,
        ),
        Strategy::Overall(overall) => per_thread_strategy(
            overall / (config.thread_no() as u32),
            &responses,
            config,
            request_mode_run,
            request_generator,
        ),
        Strategy::Duration(duration) => duration_strategy(
            *duration,
            &responses,
            config,
            request_mode_run,
            request_generator,
        ),
    }
}

fn per_thread_strategy(
    requests_per_thread: u32,
    responses: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    request_generator: &Arc<Mutex<impl RequestGenerator + Send + Sized + 'static>>,
) -> Vec<JoinHandle<()>> {
    let mut child_threads = Vec::new();
    for _ in 0..config.thread_no() {
        let responses_clone = Arc::clone(&responses);
        let request_clone = Arc::clone(&request_generator);
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..requests_per_thread {
                let request_gen = &mut *request_clone.lock().unwrap();
                let mut results = &mut *responses_clone.lock().unwrap();
                run_request(request_gen, request_mode_run, &mut results);
                thread::sleep(Duration::from_millis(config_clone.step_delay()));
            }
        });
        child_threads.push(handle);
    }
    child_threads
}

fn duration_strategy(
    duration: Duration,
    responses: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    request_generator: &Arc<Mutex<impl RequestGenerator + Send + Sized + 'static>>,
) -> Vec<JoinHandle<()>> {
    let mut child_threads = Vec::new();
    for _ in 0..config.thread_no() {
        let responses_clone = Arc::clone(&responses);
        let request_clone = Arc::clone(request_generator);
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            while start.elapsed() <= duration {
                let request_gen = &mut *request_clone.lock().unwrap();
                let results = &mut *responses_clone.lock().unwrap();
                run_request(request_gen, request_mode_run, results);
                thread::sleep(Duration::from_millis(config_clone.step_delay()));
            }
        });
        child_threads.push(handle);
    }
    child_threads
}
