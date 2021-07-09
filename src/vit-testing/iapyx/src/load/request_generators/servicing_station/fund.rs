use crate::backend::VitStationRestClient;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};

pub struct FundRequestGen {
    client: VitStationRestClient,
}

impl FundRequestGen {
    pub fn new(client: VitStationRestClient) -> Self {
        Self { client }
    }
}

impl RequestGenerator for FundRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.client
            .funds_raw()
            .map(|response| vec![])
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))
    }
}
