pub use jortestkit::load::{self, Configuration, Monitor, Request, RequestFailure};

#[derive(Clone, Debug)]
pub struct SampleRequest;

impl Request for SampleRequest {
    fn run(&self) -> Result<(), RequestFailure> {
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    }
}

#[test]
pub fn load_sanity() {
    let config = Configuration::duration(
        10,
        std::time::Duration::from_secs(1),
        50,
        Monitor::Progress(10),
    );
    load::start(SampleRequest, config, "Mock load");
}
