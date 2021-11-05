use std::{sync::Arc, time::Instant};

mod config;
mod monitor;
mod progress;
mod rayon;
mod request;
mod response;
mod stats;
mod status;

use crate::load::rayon::{DurationRequestConsumer, Executor, FixedCountRequestConsumer};
use ::rayon::iter::plumbing::bridge_unindexed;
pub use config::{Configuration, ConfigurationBuilder, Monitor, Strategy};
pub use indicatif::{MultiProgress, ProgressBar};
pub use monitor::MonitorThread;
pub use progress::{use_as_monitor_progress_bar, use_as_status_progress_bar};
pub use request::{
    Id, RayonWrapper, Request, RequestFailure, RequestGenerator, RequestSendMode, RequestStatus,
    Response,
};
use response::ResponseCollector;
pub use stats::Stats;
pub use status::{RequestStatusProvider, Status, StatusUpdaterThread};
use std::sync::mpsc::{self, Sender};

pub fn start_multi_sync<R>(request_generators: Vec<(R, Configuration, String)>) -> Vec<Stats>
where
    R: RequestGenerator + Send + 'static,
{
    let mut executors = Vec::new();
    let mut monitors = Vec::new();
    let mut collectors = Vec::new();
    let start = Instant::now();

    let m = MultiProgress::new();

    for (request_generator, config, title) in request_generators {
        let (tx, rx) = mpsc::channel();
        let response_collector = ResponseCollector::start(rx);
        let executor = run_load(request_generator, &config, RequestSendMode::Sync, tx);
        executors.push(executor);
        let pb = m.add(ProgressBar::new(1));
        monitors.push(MonitorThread::start_multi(
            response_collector.responses(),
            config.monitor().clone(),
            pb,
            &title,
        ));
        collectors.push((response_collector, title));
    }

    for ex in executors {
        ex.wait_for_finish();
    }

    let end = start.elapsed();

    for m in monitors.into_iter() {
        m.stop();
    }

    collectors
        .into_iter()
        .map(|(collector, title)| {
            let stats = Stats::new(
                Arc::try_unwrap(collector.stop())
                    .unwrap()
                    .into_inner()
                    .unwrap(),
                end,
            );
            stats.print_summary(&title);
            stats
        })
        .collect()
}

pub fn start_sync<R>(request_generator: R, config: Configuration, title: &str) -> Stats
where
    R: RequestGenerator + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let response_collector = ResponseCollector::start(rx);
    let monitor = MonitorThread::start(
        response_collector.responses(),
        config.monitor().clone(),
        title,
    );
    run_load(request_generator, &config, RequestSendMode::Sync, tx).wait_for_finish();
    let end = start.elapsed();
    monitor.stop();
    let resp = response_collector.stop();
    let stats = Stats::new(Arc::try_unwrap(resp).unwrap().into_inner().unwrap(), end);
    stats.print_summary(title);
    stats
}

pub struct BackgroundLoadProcess {
    executor: Executor,
    monitor: MonitorThread,
    response_collector: ResponseCollector,
    status_updater: StatusUpdaterThread,
    start: Instant,
}

impl BackgroundLoadProcess {
    pub fn stats(&self) -> Stats {
        let responses = self.response_collector.responses().read().unwrap().clone();
        Stats::new(responses, self.start.elapsed())
    }

    pub fn wait_for_finish(self) -> Stats {
        self.executor.wait_for_finish();

        self.monitor.stop();
        self.status_updater.stop();

        let resp = self.response_collector.stop();
        Stats::new(
            Arc::try_unwrap(resp).unwrap().into_inner().unwrap(),
            self.start.elapsed(),
        )
    }
}

pub fn start_background_async<I, S>(
    request_generator: I,
    status_provider: S,
    config: Configuration,
    title: &str,
) -> BackgroundLoadProcess
where
    I: RequestGenerator + 'static,
    S: RequestStatusProvider + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let response_collector = ResponseCollector::start(rx);
    let monitor = MonitorThread::start(
        response_collector.responses(),
        config.monitor().clone(),
        title,
    );
    let status_updater = StatusUpdaterThread::spawn(
        response_collector.responses(),
        status_provider,
        config.monitor().clone(),
        config.fetch_limit(),
        title,
        config.shutdown_grace_period(),
        config.status_pace(),
    );

    let executor = run_load(request_generator, &config, RequestSendMode::Async, tx);

    BackgroundLoadProcess {
        executor,
        monitor,
        response_collector,
        status_updater,
        start,
    }
}

pub fn start_async<I, S>(
    request_generator: I,
    status_provider: S,
    config: Configuration,
    title: &str,
) -> Stats
where
    I: RequestGenerator + 'static,
    S: RequestStatusProvider + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let response_collector = ResponseCollector::start(rx);
    let monitor = MonitorThread::start(
        response_collector.responses(),
        config.monitor().clone(),
        title,
    );
    let status_updater = StatusUpdaterThread::spawn(
        response_collector.responses(),
        status_provider,
        config.monitor().clone(),
        config.fetch_limit(),
        title,
        config.shutdown_grace_period(),
        config.status_pace(),
    );
    run_load(request_generator, &config, RequestSendMode::Async, tx).wait_for_finish();

    monitor.stop();
    status_updater.stop();

    let resp = response_collector.stop();

    let stats = Stats::new(
        Arc::try_unwrap(resp).unwrap().into_inner().unwrap(),
        start.elapsed(),
    );
    stats.print_summary(title);
    stats
}

fn run_load<R>(
    request_generator: R,
    config: &Configuration,
    request_mode_run: RequestSendMode,
    tx: Sender<Vec<Response>>,
) -> Executor
where
    R: RequestGenerator + 'static,
{
    let request_generator = RayonWrapper::from(request_generator);
    println!("Running load using {:?}", config.strategy());
    let mut executor = Executor::new(config.thread_no());
    let delay = config.step_delay();
    let strategy = config.strategy().clone();
    let thread_no = config.thread_no();
    executor.spawn(move || match strategy {
        Strategy::PerThread(per_thread_count) => bridge_unindexed(
            request_generator,
            FixedCountRequestConsumer::new(per_thread_count as u64, request_mode_run, delay, tx),
        ),
        Strategy::Overall(overall_count) => bridge_unindexed(
            request_generator,
            FixedCountRequestConsumer::new(
                overall_count as u64 * thread_no as u64,
                request_mode_run,
                delay,
                tx,
            ),
        ),
        Strategy::Duration(duration) => bridge_unindexed(
            request_generator,
            DurationRequestConsumer::new(duration, request_mode_run, delay, tx),
        ),
    });
    executor
}
