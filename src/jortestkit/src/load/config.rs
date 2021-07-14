use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Strategy {
    Duration(std::time::Duration),
    PerThread(u32),
    Overall(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Monitor {
    Standard(u64),
    Progress(u64),
    Disabled(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    thread_no: usize,
    strategy: Strategy,
    step_delay: u64,
    monitor: Monitor,
    shutdown_grace_period: u32,
    status_pace: u64,
}

impl Configuration {
    pub fn duration(
        thread_no: usize,
        duration: std::time::Duration,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
        status_pace: u64,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::Duration(duration),
            step_delay,
            monitor,
            shutdown_grace_period,
            status_pace,
        }
    }

    pub fn requests_per_thread(
        thread_no: usize,
        requests_count: u32,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
        status_pace: u64,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::PerThread(requests_count),
            step_delay,
            monitor,
            shutdown_grace_period,
            status_pace,
        }
    }

    pub fn overall_requests(
        thread_no: usize,
        requests_count: u32,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
        status_pace: u64,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::Overall(requests_count),
            step_delay,
            monitor,
            shutdown_grace_period,
            status_pace,
        }
    }

    pub fn thread_no(&self) -> usize {
        self.thread_no
    }

    pub fn strategy(&self) -> &Strategy {
        &self.strategy
    }

    pub fn step_delay(&self) -> u64 {
        self.step_delay
    }

    pub fn status_pace(&self) -> u64 {
        self.status_pace
    }

    pub fn monitor(&self) -> &Monitor {
        &self.monitor
    }

    pub fn shutdown_grace_period(&self) -> u32 {
        self.shutdown_grace_period
    }

    pub fn total_votes(&self) -> u32 {
        match self.strategy() {
            Strategy::Duration(duration) => (duration.as_millis() / self.step_delay as u128) as u32,
            Strategy::PerThread(per_thread) => self.thread_no() as u32 * per_thread,
            Strategy::Overall(overall) => *overall,
        }
    }
}
