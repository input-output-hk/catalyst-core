use std::{thread, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("timeout reached after () s")]
    TimeoutReached { secs: u64 },
}

#[derive(Clone, Debug)]
pub struct Wait {
    sleep: Duration,
    attempts: u64,
    current: u64,
}

impl Wait {
    pub fn new(sleep: Duration, attempts: u64) -> Self {
        Wait {
            sleep,
            attempts,
            current: 0u64,
        }
    }

    pub fn sleep_duration(&self) -> Duration {
        self.sleep
    }

    pub fn attempts(&self) -> u64 {
        self.attempts
    }

    pub fn current(&self) -> u64 {
        self.current
    }

    pub fn timeout_reached(&self) -> bool {
        self.current >= self.attempts
    }

    pub fn check_timeout(&self) -> Result<(), Error> {
        if self.timeout_reached() {
            return Err(Error::TimeoutReached {
                secs: self.sleep.as_secs() * self.attempts,
            });
        }
        Ok(())
    }

    pub fn advance(&mut self) {
        self.current += 1;
        thread::sleep(self.sleep);
    }
}

impl Default for Wait {
    fn default() -> Self {
        Wait::new(Duration::from_secs(1), 5)
    }
}

impl From<Duration> for Wait {
    fn from(duration: Duration) -> Wait {
        let attempts = duration.as_secs();
        let sleep = 1;
        Wait::new(Duration::from_secs(sleep), attempts)
    }
}

pub struct WaitBuilder {
    sleep: Duration,
    attempts: u64,
}

impl Default for WaitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WaitBuilder {
    pub fn new() -> Self {
        WaitBuilder {
            sleep: Duration::from_secs(1),
            attempts: 5,
        }
    }

    pub fn tries(&mut self, attempts: u64) -> &mut Self {
        self.attempts = attempts;
        self
    }

    pub fn sleep_between_tries(&mut self, sleep: u64) -> &mut Self {
        self.sleep = Duration::from_secs(sleep);
        self
    }

    pub fn build(&self) -> Wait {
        Wait::new(self.sleep, self.attempts)
    }
}
