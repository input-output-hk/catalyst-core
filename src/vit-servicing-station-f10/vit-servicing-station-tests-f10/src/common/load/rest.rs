use crate::common::clients::RestClient;
use crate::common::data::Snapshot as Data;
use crate::common::load::SnapshotRandomizer;
use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct VitRestRequestGenerator {
    rest_client: RestClient,
    snapshot_randomizer: SnapshotRandomizer,
}

impl VitRestRequestGenerator {
    pub fn new(snapshot: Data, mut rest_client: RestClient) -> Self {
        rest_client.disable_log();

        Self {
            snapshot_randomizer: SnapshotRandomizer::new(snapshot),
            rest_client,
        }
    }
}

impl RequestGenerator for VitRestRequestGenerator {
    fn next(&mut self) -> std::result::Result<Request, RequestFailure> {
        self.rest_client
            .set_api_token(self.snapshot_randomizer.random_token());

        match self.snapshot_randomizer.random_usize() % 3 {
            0 => self
                .rest_client
                .health()
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Health: {}", e))),
            1 => self
                .rest_client
                .proposals()
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Proposals: {}", e))),
            2 => self
                .rest_client
                .proposal(&self.snapshot_randomizer.random_proposal_id().to_string())
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Proposals by id: {}", e))),
            3 => self
                .rest_client
                .fund(&self.snapshot_randomizer.random_fund_id().to_string())
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Funds by id: {}", e))),
            _ => unreachable!(),
        }
    }

    fn split(self) -> (Self, Option<Self>) {
        todo!()
    }
}
