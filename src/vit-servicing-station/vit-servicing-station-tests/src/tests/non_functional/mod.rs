use crate::common::{clients::RestClient, startup::quick_start};
use assert_fs::TempDir;

use crate::common::data::Snapshot;
use jortestkit::load::{self, Configuration, Id, Monitor, RequestFailure, RequestGenerator};
use rand_core::{OsRng, RngCore};

#[derive(Clone, Debug)]
struct SnapshotRandomizer {
    snapshot: Snapshot,
    random: OsRng,
}

#[derive(Clone, Debug)]
struct VitRestRequestGenerator {
    rest_client: RestClient,
    snapshot_randomizer: SnapshotRandomizer,
}

impl SnapshotRandomizer {
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            random: OsRng,
        }
    }

    pub fn random_token(&mut self) -> String {
        let tokens = self.snapshot.tokens();
        let random_idx = self.random_usize() % tokens.len();
        tokens
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .get(random_idx)
            .unwrap()
            .clone()
    }

    pub fn random_usize(&mut self) -> usize {
        self.random.next_u32() as usize
    }

    pub fn random_proposal_id(&mut self) -> i32 {
        let proposals = self.snapshot.proposals();
        let random_idx = self.random_usize() % proposals.len();
        proposals
            .get(random_idx)
            .unwrap()
            .proposal
            .internal_id
            .clone()
    }

    pub fn random_fund_id(&mut self) -> i32 {
        let funds = self.snapshot.funds();
        let random_idx = self.random_usize() % funds.len();
        funds.get(random_idx).unwrap().id.clone()
    }
}

impl VitRestRequestGenerator {
    pub fn new(snapshot: Snapshot, mut rest_client: RestClient) -> Self {
        rest_client.disable_log();

        Self {
            snapshot_randomizer: SnapshotRandomizer::new(snapshot),
            rest_client,
        }
    }
}

impl RequestGenerator for VitRestRequestGenerator {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.rest_client
            .set_api_token(self.snapshot_randomizer.random_token());

        match self.snapshot_randomizer.random_usize() % 7 {
            0 => self
                .rest_client
                .health()
                .map(|_| vec![Option::None])
                .map_err(|e| RequestFailure::General(format!("Health: {}", e.to_string()))),
            1 => self
                .rest_client
                .proposals()
                .map(|_| vec![Option::None])
                .map_err(|e| RequestFailure::General(format!("Proposals: {}", e.to_string()))),
            2 => self
                .rest_client
                .proposal(&self.snapshot_randomizer.random_proposal_id().to_string())
                .map(|_| vec![Option::None])
                .map_err(|e| {
                    RequestFailure::General(format!("Proposals by id: {}", e.to_string()))
                }),
            3 => self
                .rest_client
                .fund(&self.snapshot_randomizer.random_fund_id().to_string())
                .map(|_| vec![Option::None])
                .map_err(|e| RequestFailure::General(format!("Funds by id: {}", e.to_string()))),
            _ => unreachable!(),
        }
    }
}

#[test]
pub fn rest_load_quick() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = Configuration::duration(
        10,
        std::time::Duration::from_secs(40),
        500,
        Monitor::Progress(100),
        0,
    );
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}

#[test]
pub fn rest_load_long() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = Configuration::duration(
        3,
        std::time::Duration::from_secs(18_000),
        1_000,
        Monitor::Progress(10_000),
        0,
    );
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}
