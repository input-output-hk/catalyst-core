use std::time::{Duration, Instant};
use thiserror::Error;
pub type Response = Result<Duration, RequestFailure>;

#[derive(Error, Debug, Clone)]
pub enum RequestFailure {
    #[error("failure during request execution {0}")]
    General(String),
}

pub trait Request {
    fn run(&self) -> Result<(), RequestFailure>;
}

pub struct RequestWithTimer;

impl RequestWithTimer {
    pub fn run(request: impl Request) -> Response {
        let start = Instant::now();
        request.run()?;
        Ok(start.elapsed())
    }
}
