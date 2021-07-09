use crate::backend::VitStationRestClient;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};

pub struct ChallengeRequestGen {
    client: VitStationRestClient,
}

impl ChallengeRequestGen {
    pub fn new(client: VitStationRestClient) -> Self {
        Self { client }
    }
}

impl RequestGenerator for ChallengeRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.client
            .challenges_raw()
            .map(|response| vec![])
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))
    }
}
