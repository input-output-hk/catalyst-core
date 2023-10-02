use jortestkit::load::{Id, RequestStatusProvider, Status};
use std::time::Duration;

mod short;
#[cfg(feature = "soak")]
mod soak;

struct MockStatusProvider;

impl RequestStatusProvider for MockStatusProvider {
    fn get_statuses(&self, ids: &[Id]) -> Vec<Status> {
        ids.iter()
            .map(|id| Status::new_success(Duration::from_millis(10), id.clone()))
            .collect()
    }
}
