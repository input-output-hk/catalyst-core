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
pub use indicatif::{MultiProgress, ProgressBar};
pub use monitor::MonitorThread;
pub use progress::{use_as_monitor_progress_bar, use_as_status_progress_bar};
pub use request::{Id, RequestFailure, RequestGenerator, RequestSendMode, RequestStatus, Response};
pub use stats::Stats;
pub use status::{RequestStatusProvider, Status, StatusUpdaterThread};

pub fn start_multi_sync<R>(request_generators: Vec<(R, Configuration, String)>) -> Vec<Stats>
where
    R: RequestGenerator + Send + 'static,
{
    let mut child_threads = Vec::new();
    let mut monitors = Vec::new();
    let mut multi_responses = Vec::new();
    let start = Instant::now();

    let m = MultiProgress::new();

    for (request_generator, config, title) in request_generators {
        let responses = Arc::new(Mutex::new(Vec::new()));
        let request_generator = Arc::new(Mutex::new(request_generator));
        let threads = get_threads(
            &request_generator,
            &config,
            RequestSendMode::Sync,
            &responses,
        );
        multi_responses.push((responses.clone(), title.clone()));
        let pb = m.add(ProgressBar::new(1));
        monitors.push(MonitorThread::start_multi(
            &responses,
            config.monitor().clone(),
            pb,
            &title,
        ));
        child_threads.extend(threads);
    }

    m.join_and_clear().unwrap();

    for t in child_threads.into_iter() {
        let _child_threads = t.join();
    }

    for m in monitors.into_iter() {
        m.stop();
    }

    multi_responses
        .into_iter()
        .map(|(resps, title)| {
            let lock_request = &mut resps.lock().unwrap();
            let stats = Stats::new(lock_request.to_vec(), start.elapsed());
            stats.print_summary(&title);
            stats
        })
        .collect()
}

pub fn start_sync<R>(request_generator: R, config: Configuration, title: &str) -> Stats
where
    R: RequestGenerator + Send + 'static,
{
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

pub fn start_background_async<R, S>(
    request_generator: R,
    status_provider: S,
    config: Configuration,
    title: &str,
) -> BackgroundLoadProcess
where
    R: RequestGenerator + Send + 'static,
    S: RequestStatusProvider + Send + 'static,
{
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
        config.status_pace(),
    );

    BackgroundLoadProcess {
        responses,
        threads: child_threads,
        monitor,
        status_updater,
        start,
    }
}

pub fn start_async<R, S>(
    request_generator: R,
    status_provider: S,
    config: Configuration,
    title: &str,
) -> Stats
where
    R: RequestGenerator + Send + 'static,
    S: RequestStatusProvider + Send + 'static,
{
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
        config.status_pace(),
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

fn get_threads<R>(
    request_generator: &Arc<Mutex<R>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    responses: &Arc<Mutex<Vec<Response>>>,
) -> Vec<JoinHandle<()>>
where
    R: RequestGenerator + Send + 'static,
{
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

fn per_thread_strategy<R>(
    requests_per_thread: u32,
    responses: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    request_generator: &Arc<Mutex<R>>,
) -> Vec<JoinHandle<()>>
where
    R: RequestGenerator + Send + 'static,
{
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

fn duration_strategy<R>(
    duration: Duration,
    responses: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    request_generator: &Arc<Mutex<R>>,
) -> Vec<JoinHandle<()>>
where
    R: RequestGenerator + Send + 'static,
{
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
