use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use thread::JoinHandle;

mod config;
mod monitor;
mod request;
mod stats;

pub use config::{Configuration, Monitor, Strategy};
pub use monitor::MonitorThread;
pub use request::{Request, RequestFailure, RequestWithTimer, Response};
pub use stats::Stats;

pub fn start(
    request: impl Request + std::clone::Clone + Send + Sized + 'static,
    config: Configuration,
    title: &str,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let start = Instant::now();
    let child_threads = get_threads(&request, &config, &requests);
    let monitor = MonitorThread::start(&requests, config.monitor().clone(), title);

    for t in child_threads.into_iter() {
        let _child_threads = t.join();
    }
    monitor.stop();

    let lock_request = &mut requests.lock().unwrap();
    let stats = Stats::new(lock_request.to_vec(), start.elapsed());
    stats.print_summary(title);
}

fn get_threads(
    request: &(impl Request + std::clone::Clone + Send + Sized + 'static),
    config: &Configuration,
    requests: &Arc<Mutex<Vec<Response>>>,
) -> Vec<JoinHandle<()>> {
    println!("Running load using {:?}", config.strategy());
    match config.strategy() {
        Strategy::PerThread(per_thread) => {
            per_thread_strategy(*per_thread, &requests, config, request)
        }
        Strategy::Overall(overall) => per_thread_strategy(
            overall / (config.thread_no() as u32),
            &requests,
            config,
            request,
        ),
        Strategy::Duration(duration) => duration_strategy(*duration, &requests, config, request),
    }
}

fn per_thread_strategy(
    requests_per_thread: u32,
    requests: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request: &(impl Request + std::clone::Clone + Send + Sized + 'static),
) -> Vec<JoinHandle<()>> {
    let mut child_threads = Vec::new();
    for _ in 0..config.thread_no() {
        let requests_clone = Arc::clone(&requests);
        let request_clone = request.clone();
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            for _ in 0..requests_per_thread {
                let result = RequestWithTimer::run(request_clone.clone());
                requests_clone.lock().unwrap().push(result);
                thread::sleep(Duration::from_millis(config_clone.step_delay()));
            }
        });
        child_threads.push(handle);
    }
    child_threads
}

fn duration_strategy(
    duration: Duration,
    requests: &Arc<Mutex<Vec<Response>>>,
    config: &Configuration,
    request: &(impl Request + std::clone::Clone + Send + Sized + 'static),
) -> Vec<JoinHandle<()>> {
    let mut child_threads = Vec::new();
    for _ in 0..config.thread_no() {
        let requests_clone = Arc::clone(&requests);
        let request_clone = request.clone();
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            while start.elapsed() <= duration {
                let result = RequestWithTimer::run(request_clone.clone());
                requests_clone.lock().unwrap().push(result);
                thread::sleep(Duration::from_millis(config_clone.step_delay()));
            }
        });
        child_threads.push(handle);
    }
    child_threads
}
