pub use jortestkit::load::{
    self, Configuration, Id, Monitor, RequestFailure, RequestGenerator, RequestStatus,
    RequestStatusProvider, Response,
};
use load::Status;

#[derive(Clone, Debug)]
pub struct SampleRequestGenerator {
    counter: u32,
}

impl RequestGenerator for SampleRequestGenerator {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.counter = self.counter + 1;
        Ok(vec![None])
    }
}

#[test]
pub fn load_sanity_sync() {
    let config = Configuration::duration(
        1,
        std::time::Duration::from_secs(3),
        50,
        Monitor::Progress(10),
        0,
        1,
    );
    load::start_sync(SampleRequestGenerator { counter: 1 }, config, "Mock load");
}

#[derive(Clone, Debug)]
pub struct AsyncSampleRequestGenerator {
    counter: u32,
}

impl RequestGenerator for AsyncSampleRequestGenerator {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let id = self.counter.to_string();
        self.counter = self.counter + 1;
        Ok(vec![Some(id)])
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
    let config = Configuration::duration(
        1,
        std::time::Duration::from_secs(3),
        50,
        Monitor::Progress(10),
        0,
        1,
    );
    load::start_async(
        AsyncSampleRequestGenerator { counter: 1 },
        SampleRequestStatusProvider,
        config,
        "Mock async load",
    );
}
