use super::request::Response;
use std::time::Duration;

pub struct Stats {
    requests: Vec<Response>,
    duration: Duration,
}

impl Stats {
    pub fn new(requests: Vec<Response>, duration: Duration) -> Self {
        Self { requests, duration }
    }

    pub fn calculate_mean(&self) -> f64 {
        let total_requests = self.requests.len() as f64;
        let total_duration: f64 = self
            .requests
            .iter()
            .filter(|r| r.is_ok())
            .map(|r| r.as_ref().unwrap().as_secs_f64())
            .sum();

        total_duration / total_requests as f64
    }

    pub fn total_requests_made(&self) -> usize {
        self.requests.len()
    }

    pub fn calculate_tps(&self) -> f64 {
        (self.total_requests_made() as f64) / self.duration.as_secs_f64()
    }

    pub fn print_summary(&self, title: &str) {
        let mean = self.calculate_mean();
        let requests = self.total_requests_made();
        let tps = self.calculate_tps();
        println!("Load scenario `{}` finished", title);
        println!("I made a total of {:.2} requests, the mean response time was: {:.3} seconds. tps: {:.2}. Test duration: {} s", requests, mean, tps, self.duration.as_secs());
    }

    pub fn tps_status(&self) -> String {
        format!(
            "tps: {:.2}, requests sent: {:.2}, duration: {} s",
            self.calculate_tps(),
            self.total_requests_made(),
            self.duration.as_secs()
        )
    }
}
