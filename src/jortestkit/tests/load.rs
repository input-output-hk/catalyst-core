use jortestkit::load::ConfigurationBuilder;
pub use jortestkit::load::{
    self, Configuration, Id, Monitor, Request, RequestFailure, RequestGenerator, RequestStatus,
    RequestStatusProvider, Response,
};
use load::Status;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SampleRequestGenerator {
    counter: u64,
}

impl RequestGenerator for SampleRequestGenerator {
    fn next(&mut self) -> Result<Request, RequestFailure> {
        std::thread::sleep(Duration::from_millis(100));
        self.counter += 1;
        Ok(Request {
            ids: vec![None],
            duration: Duration::ZERO,
        })
    }

    fn split(self) -> (Self, Option<Self>) {
        (self, None)
    }
}

#[test]
pub fn load_sanity_sync() {
    let config = ConfigurationBuilder::duration(Duration::from_secs(3))
        .step_delay(Duration::from_millis(50))
        .monitor(Monitor::Progress(10))
        .build();

    load::start_sync(SampleRequestGenerator { counter: 1 }, config, "Mock load");
}

#[test]
pub fn load_sanity_multi_sync() {
    let config = ConfigurationBuilder::duration(Duration::from_secs(5))
        .thread_no(5)
        .step_delay(Duration::from_millis(10))
        .monitor(Monitor::Progress(100))
        .build();

    load::start_multi_sync(vec![
        (
            SampleRequestGenerator { counter: 1 },
            config.clone(),
            "Mock multi load #1".to_string(),
        ),
        (
            SampleRequestGenerator { counter: 1 },
            config.clone(),
            "Mock multi load #2".to_string(),
        ),
        (
            SampleRequestGenerator { counter: 1 },
            config,
            "Mock multi load #3".to_string(),
        ),
    ]);
}

#[derive(Clone, Debug)]
pub struct AsyncSampleRequestGenerator {
    counter: u32,
}

impl RequestGenerator for AsyncSampleRequestGenerator {
    fn next(&mut self) -> Result<Request, RequestFailure> {
        std::thread::sleep(Duration::from_millis(100));
        let id = self.counter.to_string();
        self.counter += 1;
        Ok(Request {
            ids: vec![Some(id)],
            duration: Duration::ZERO,
        })
    }

    fn split(self) -> (Self, Option<Self>) {
        (self, None)
    }
}

#[derive(Clone, Debug)]
pub struct SampleRequestStatusProvider;

impl RequestStatusProvider for SampleRequestStatusProvider {
    fn get_statuses(&self, ids: &[Id]) -> Vec<Status> {
        ids.iter()
            .map(|id| Status::new_pending(std::time::Duration::from_secs(5), id.clone()))
            .collect()
    }
}

#[test]
pub fn load_sanity_async() {
    let config = ConfigurationBuilder::duration(Duration::from_secs(3))
        .step_delay(Duration::from_millis(50))
        .monitor(Monitor::Progress(10))
        .build();
    load::start_async(
        AsyncSampleRequestGenerator { counter: 1 },
        SampleRequestStatusProvider,
        config,
        "Mock async load",
    );
}
