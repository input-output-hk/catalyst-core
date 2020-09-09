#[derive(Debug, Clone)]
pub enum Strategy {
    Duration(std::time::Duration),
    PerThread(u32),
    Overall(u32),
}

#[derive(Debug, Clone)]
pub enum Monitor {
    Standard(u64),
    Progress(u64),
    Disabled(u64),
}

#[derive(Debug, Clone)]
pub struct Configuration {
    thread_no: usize,
    strategy: Strategy,
    step_delay: u64,
    monitor: Monitor,
    shutdown_grace_period: u32,
}

impl Configuration {
    pub fn duration(
        thread_no: usize,
        duration: std::time::Duration,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::Duration(duration),
            step_delay,
            monitor,
            shutdown_grace_period,
        }
    }

    pub fn requests_per_thread(
        thread_no: usize,
        requests_count: u32,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::PerThread(requests_count),
            step_delay,
            monitor,
            shutdown_grace_period,
        }
    }

    pub fn overall_requests(
        thread_no: usize,
        requests_count: u32,
        step_delay: u64,
        monitor: Monitor,
        shutdown_grace_period: u32,
    ) -> Configuration {
        Self {
            thread_no,
            strategy: Strategy::Overall(requests_count),
            step_delay,
            monitor,
            shutdown_grace_period,
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

    pub fn monitor(&self) -> &Monitor {
        &self.monitor
    }

    pub fn shutdown_grace_period(&self) -> u32 {
        self.shutdown_grace_period
    }
}
