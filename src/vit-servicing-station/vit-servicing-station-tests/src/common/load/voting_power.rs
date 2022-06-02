use crate::common::clients::RestClient;
use crate::common::snapshot::Snapshot;
use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use rand::Rng;
use rand::RngCore;
use std::time::Duration;

const DEFAULT_MAX_SPLITS: usize = 7; // equals to 128 splits, will likely not reach that value but it's there just to prevent a stack overflow

#[derive(Clone, Debug)]
pub struct VotingPowerRequestGenerator {
    rest_client: RestClient,
    snapshot: Snapshot,
    max_splits: usize, // avoid infinite splitting
}

impl VotingPowerRequestGenerator {
    pub fn new(snapshot: Snapshot, mut rest_client: RestClient) -> Self {
        rest_client.disable_log();

        Self {
            snapshot,
            rest_client,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }
}

impl RequestGenerator for VotingPowerRequestGenerator {
    fn next(&mut self) -> std::result::Result<Request, RequestFailure> {
        let mut rng = rand::rngs::OsRng;
        let content = &self.snapshot.content;
        self.rest_client
            .voting_power(
                &self.snapshot.tag,
                &content[rng.gen_range(0, content.len())].voting_key.to_hex(),
            )
            .map(|_| Request {
                ids: vec![Some(rng.next_u64().to_string())],
                duration: Duration::ZERO,
            })
            .map_err(|e| RequestFailure::General(format!("voting power: {}", e)))
    }

    fn split(mut self) -> (Self, Option<Self>) {
        // Since VotingPowerRequestGenerator doesn't need any requests house keeping after sending
        // we could split as many times as we want
        // but that may trigger a bug in rayon so we artificially limit it
        if self.max_splits == 0 {
            return (self, None);
        }
        self.max_splits -= 1;
        (self.clone(), Some(self))
    }
}
