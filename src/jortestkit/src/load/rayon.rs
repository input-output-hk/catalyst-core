use super::{request::Request, RequestFailure, RequestSendMode, Response};
use rayon::iter::plumbing::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::cell::Cell;
use std::ops::FnOnce;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    mpsc::Sender,
    Arc,
};
use std::time::{Duration, Instant};

type Req = Result<Request, RequestFailure>;

#[derive(Clone)]
struct RateLimiter {
    delay: Duration,
    last_tick: Instant,
}

impl RateLimiter {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            last_tick: Instant::now() - delay,
        }
    }

    /// Wait at most for `delay`, but account for time passed in between successive requests
    fn rate_adjust(&mut self) {
        std::thread::sleep(self.delay.saturating_sub(self.last_tick.elapsed()));
        self.last_tick = Instant::now();
    }
}

#[derive(Clone)]
pub struct FixedCountRequestConsumer {
    count: Cell<u64>,
    tx: Sender<Vec<Response>>,
    request_mode: RequestSendMode,
    rate: RateLimiter,
}

#[derive(Clone)]
pub struct DurationRequestConsumer {
    until: Instant,
    tx: Sender<Vec<Response>>,
    request_mode: RequestSendMode,
    rate: RateLimiter,
}

impl FixedCountRequestConsumer {
    pub fn new(
        count: u64,
        request_mode: RequestSendMode,
        delay: Duration,
        tx: Sender<Vec<Response>>,
    ) -> Self {
        Self {
            count: Cell::new(count),
            tx,
            request_mode,
            rate: RateLimiter::new(delay),
        }
    }
}

impl DurationRequestConsumer {
    pub fn new(
        duration: Duration,
        request_mode: RequestSendMode,
        delay: Duration,
        tx: Sender<Vec<Response>>,
    ) -> Self {
        Self {
            until: Instant::now() + duration,
            tx,
            request_mode,
            rate: RateLimiter::new(delay),
        }
    }
}

pub struct NoopReducer;
impl Reducer<()> for NoopReducer {
    fn reduce(self, _left: (), _right: ()) {}
}

impl Consumer<Req> for FixedCountRequestConsumer {
    type Reducer = NoopReducer;
    type Folder = Self;
    type Result = ();

    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
        let split = self.count.get() - index as u64;
        self.count.set(index as u64);
        (
            Self {
                count: Cell::new(split),
                ..self.clone()
            },
            self,
            NoopReducer,
        )
    }
    fn into_folder(self) -> Self::Folder {
        self
    }
    fn full(&self) -> bool {
        false
    }
}

impl Consumer<Req> for DurationRequestConsumer {
    type Reducer = NoopReducer;
    type Folder = Self;
    type Result = ();

    fn split_at(self, _index: usize) -> (Self, Self, Self::Reducer) {
        (self.clone(), self, NoopReducer)
    }
    fn into_folder(self) -> Self::Folder {
        self
    }
    fn full(&self) -> bool {
        false
    }
}

impl Folder<Req> for FixedCountRequestConsumer {
    type Result = ();

    fn consume(mut self, item: Req) -> Self {
        self.rate.rate_adjust();
        process_request(item, self.request_mode, &self.tx);
        self.count.set(self.count.get() - 1);
        self
    }
    fn complete(self) -> Self::Result {}

    fn full(&self) -> bool {
        self.count.get() == 0
    }
}

impl Folder<Req> for DurationRequestConsumer {
    type Result = ();

    fn consume(mut self, item: Req) -> Self {
        self.rate.rate_adjust();
        process_request(item, self.request_mode, &self.tx);
        self
    }
    fn complete(self) -> Self::Result {}

    fn full(&self) -> bool {
        Instant::now() > self.until
    }
}

fn process_request(req: Req, request_mode: RequestSendMode, tx: &Sender<Vec<Response>>) {
    match req {
        Ok(Request { ids, duration }) => tx
            .send(
                ids.into_iter()
                    .map(|id| match request_mode {
                        RequestSendMode::Sync => Response::success(id, duration),
                        RequestSendMode::Async => Response::pending(id, duration),
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        Err(failure) => tx
            .send(vec![Response::failure(None, failure, Duration::ZERO)])
            .unwrap(),
    };
}

impl UnindexedConsumer<Req> for FixedCountRequestConsumer {
    fn split_off_left(&self) -> Self {
        let cur = self.count.get();
        let mid = cur / 2;
        self.count.set(cur - mid);

        Self {
            count: Cell::new(mid),
            ..self.clone()
        }
    }
    fn to_reducer(&self) -> <Self as Consumer<Req>>::Reducer {
        NoopReducer
    }
}

impl UnindexedConsumer<Req> for DurationRequestConsumer {
    fn split_off_left(&self) -> Self {
        self.clone()
    }
    fn to_reducer(&self) -> <Self as Consumer<Req>>::Reducer {
        NoopReducer
    }
}

pub struct Executor {
    task_completed: Arc<AtomicU8>,
    task_spawned: u8,
    thread_pool: ThreadPool,
}

impl Executor {
    pub fn new(n_threads: usize) -> Self {
        Self {
            task_completed: Arc::new(AtomicU8::new(0)),
            task_spawned: 0,
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(n_threads)
                .thread_name(|n| n.to_string())
                .build()
                .unwrap(),
        }
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_spawned += 1;
        let task_completed = self.task_completed.clone();
        self.thread_pool.spawn(move || {
            f();
            task_completed.fetch_add(1, Ordering::SeqCst);
        });
    }

    pub fn wait_for_finish(self) {
        while self.task_completed.load(Ordering::SeqCst) != self.task_spawned {
            std::thread::park_timeout(Duration::from_millis(100));
        }
    }
}
