use super::request::{RequestFailure, Response};
use crate::prelude::{EfficiencyBenchmarkDef, EfficiencyBenchmarkFinish};
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
            .map(|r| r.duration().as_secs_f64())
            .sum();

        total_duration / total_requests as f64
    }

    pub fn total_requests_made(&self) -> usize {
        self.requests.len()
    }

    pub fn calculate_tps(&self) -> f64 {
        (self.total_requests_made() as f64) / self.duration.as_secs_f64()
    }

    pub fn measure(&self, measurement_name: &str, target: u32) -> EfficiencyBenchmarkFinish {
        EfficiencyBenchmarkDef::new(measurement_name.to_string())
            .target(target)
            .start()
            .increment_by(self.calculate_passrate() as u32)
            .stop()
    }

    pub fn print_summary(&self, title: &str) {
        let mean = self.calculate_mean();
        let requests = self.total_requests_made();
        let tps = self.calculate_tps();
        let passrate = self.calculate_passrate();
        println!("Load scenario `{}` finished", title);
        println!("I made a total of {:.2} requests ({} passed/ {} failed/ {} pending), the mean response time was: {:.3} seconds. tps: {:.2}. Test duration: {} s. Passrate: {} %", 
            requests,
            self.total_requests_passed(),
            self.total_requests_failed(),
            self.total_requests_pending(),
            mean, tps, self.duration.as_secs(),passrate);
        self.print_errors_if_any();
    }

    pub fn calculate_passrate(&self) -> f64 {
        ((self.total_requests_passed() as f64) / self.total_requests_made() as f64) * 100.0
    }

    pub fn total_requests_passed(&self) -> usize {
        self.requests.iter().filter(|r| r.is_success()).count()
    }

    pub fn total_requests_failed(&self) -> usize {
        self.requests.iter().filter(|r| r.is_failed()).count()
    }

    pub fn total_requests_pending(&self) -> usize {
        self.requests.iter().filter(|r| r.is_pending()).count()
    }

    pub fn print_errors_if_any(&self) {
        let errors = self.errors();
        if errors.is_empty() {
            return;
        }

        if errors.len() > 10 {
            println!("{} errors fund. Printing only 10 :", errors.len());
            let first_10_errors: Vec<RequestFailure> = errors.iter().cloned().take(10).collect();
            self.print_errors(first_10_errors);
            return;
        }
        println!("{} errors fund", errors.len());
        self.print_errors(errors);
    }

    fn print_errors(&self, errors: Vec<RequestFailure>) {
        for (id, error) in errors.iter().enumerate() {
            println!("{}: {}", id + 1, error);
        }
    }

    pub fn errors(&self) -> Vec<RequestFailure> {
        self.requests
            .iter()
            .cloned()
            .filter(|r| r.is_err())
            .map(|r| r.err().unwrap())
            .collect()
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
