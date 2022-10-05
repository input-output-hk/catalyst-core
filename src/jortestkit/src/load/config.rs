use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Strategy {
    Duration(std::time::Duration),
    Overall(u32),
    PerThread(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Monitor {
    Standard(u64),
    Progress(u64),
    Disabled(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    fetch_limit: Option<usize>,
    monitor: Monitor,
    shutdown_grace_period: Duration,
    status_pace: Duration,
    step_delay: Duration,
    strategy: Strategy,
    thread_no: usize,
}

impl Configuration {
    pub fn fetch_limit(&self) -> Option<usize> {
        self.fetch_limit
    }

    pub fn monitor(&self) -> &Monitor {
        &self.monitor
    }

    pub fn shutdown_grace_period(&self) -> Duration {
        self.shutdown_grace_period
    }
    pub fn status_pace(&self) -> Duration {
        self.status_pace
    }

    pub fn step_delay(&self) -> Duration {
        self.step_delay
    }

    pub fn strategy(&self) -> &Strategy {
        &self.strategy
    }

    pub fn thread_no(&self) -> usize {
        self.thread_no
    }

    pub fn total_votes(&self) -> u32 {
        match self.strategy() {
            Strategy::Duration(duration) => {
                (duration.as_millis() / self.step_delay.as_millis()) as u32
            }
            Strategy::PerThread(per_thread) => self.thread_no() as u32 * per_thread,
            Strategy::Overall(overall) => *overall,
        }
    }
}

pub struct ConfigurationBuilder {
    fetch_limit: Option<usize>,
    monitor: Monitor,
    shutdown_grace_period: Duration,
    status_pace: Duration,
    step_delay: Duration,
    strategy: Strategy,
    thread_no: usize,
}

impl ConfigurationBuilder {
    pub fn duration(duration: Duration) -> Self {
        Self {
            fetch_limit: None,
            monitor: Monitor::Disabled(100),
            shutdown_grace_period: Duration::ZERO,
            status_pace: Duration::from_secs(1),
            step_delay: Duration::from_millis(100),
            strategy: Strategy::Duration(duration),
            thread_no: 1,
        }
    }

    pub fn overall_requests(n_requests: u32) -> Self {
        Self {
            fetch_limit: None,
            monitor: Monitor::Disabled(100),
            shutdown_grace_period: Duration::ZERO,
            status_pace: Duration::from_secs(1),
            step_delay: Duration::from_millis(100),
            strategy: Strategy::Overall(n_requests),
            thread_no: 1,
        }
    }

    pub fn requests_per_thread(n_requests: u32) -> Self {
        Self {
            fetch_limit: None,
            monitor: Monitor::Disabled(100),
            shutdown_grace_period: Duration::ZERO,
            status_pace: Duration::from_secs(1),
            step_delay: Duration::from_millis(100),
            strategy: Strategy::PerThread(n_requests),
            thread_no: 1,
        }
    }

    pub fn fetch_limit(self, fetch_limit: usize) -> Self {
        Self {
            fetch_limit: Some(fetch_limit),
            ..self
        }
    }

    pub fn monitor(self, monitor: Monitor) -> Self {
        Self { monitor, ..self }
    }

    pub fn shutdown_grace_period(self, shutdown_grace_period: Duration) -> Self {
        Self {
            shutdown_grace_period,
            ..self
        }
    }

    pub fn status_pace(self, status_pace: Duration) -> Self {
        Self {
            status_pace,
            ..self
        }
    }

    pub fn step_delay(self, step_delay: Duration) -> Self {
        Self { step_delay, ..self }
    }

    pub fn thread_no(self, thread_no: usize) -> Self {
        Self { thread_no, ..self }
    }

    pub fn build(self) -> Configuration {
        Configuration {
            fetch_limit: self.fetch_limit,
            monitor: self.monitor,
            shutdown_grace_period: self.shutdown_grace_period,
            status_pace: self.status_pace,
            step_delay: self.step_delay,
            strategy: self.strategy,
            thread_no: self.thread_no,
        }
    }
}
