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
    limiter: Option<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl<T: HttpClient> RateLimitClient<T> {
    pub fn new(inner: T, request_interval_ms: u64) -> Self {
        let limiter = if request_interval_ms == 0 {
            None
        } else {
            let quota = Quota::with_period(Duration::from_millis(request_interval_ms)).unwrap();
            Some(RateLimiter::direct(quota))
        };
        Self { inner, limiter }
    }
}

impl<T: HttpClient> HttpClient for RateLimitClient<T> {
    fn get<S>(&self, path: &str) -> Result<HttpResponse<S>, Report>
    where
        S: for<'a> Deserialize<'a>,
    {
        if let Some(limiter) = &self.limiter {
            while let Err(e) = limiter.check() {
                let time = e.wait_time_from(Instant::now());
                debug!("waiting for rate limit: {time:?}");
                std::thread::sleep(time);
            }
        }
        self.inner.get(path)
    }
}
