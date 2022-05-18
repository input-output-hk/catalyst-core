use std::time::{Duration, Instant};

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

use color_eyre::Report;
use log::debug;
use serde::Deserialize;

use super::{HttpClient, HttpResponse};

pub struct RateLimitClient<T: HttpClient> {
    inner: T,
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl<T: HttpClient> RateLimitClient<T> {
    pub fn new(inner: T, request_interval_ms: u64) -> Self {
        let quota = Quota::with_period(Duration::from_millis(request_interval_ms)).unwrap();
        let limiter = RateLimiter::direct(quota);
        Self { inner, limiter }
    }
}

impl<T: HttpClient> HttpClient for RateLimitClient<T> {
    fn get<S>(&self, path: &str) -> Result<HttpResponse<S>, Report>
    where
        S: for<'a> Deserialize<'a>,
    {
        while let Err(e) = self.limiter.check() {
            let time = e.wait_time_from(Instant::now());
            debug!("waiting for {time:?}");
            std::thread::sleep(time);
        }
        self.inner.get(path)
    }
}
