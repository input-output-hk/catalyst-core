use crate::common::{clients::RestClient, data::Snapshot, startup::quick_start};
use assert_fs::TempDir;
use jortestkit::load::{
    self, ConfigurationBuilder, Monitor, Request, RequestFailure, RequestGenerator,
};
use rand_core::{OsRng, RngCore};
use std::time::Duration;

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
        proposals.get(random_idx).unwrap().proposal.internal_id
    }

    pub fn random_fund_id(&mut self) -> i32 {
        let funds = self.snapshot.funds();
        let random_idx = self.random_usize() % funds.len();
        funds.get(random_idx).unwrap().id
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
    fn next(&mut self) -> std::result::Result<Request, RequestFailure> {
        self.rest_client
            .set_api_token(self.snapshot_randomizer.random_token());

        match self.snapshot_randomizer.random_usize() % 7 {
            0 => self
                .rest_client
                .health()
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Health: {}", e.to_string()))),
            1 => self
                .rest_client
                .proposals()
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Proposals: {}", e.to_string()))),
            2 => self
                .rest_client
                .proposal(&self.snapshot_randomizer.random_proposal_id().to_string())
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| {
                    RequestFailure::General(format!("Proposals by id: {}", e.to_string()))
                }),
            3 => self
                .rest_client
                .fund(&self.snapshot_randomizer.random_fund_id().to_string())
                .map(|_| Request {
                    ids: vec![Option::None],
                    duration: Duration::ZERO,
                })
                .map_err(|e| RequestFailure::General(format!("Funds by id: {}", e.to_string()))),
            _ => unreachable!(),
        }
    }

    fn split(self) -> (Self, Option<Self>) {
        todo!()
    }
}

#[test]
pub fn rest_load_quick() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(40))
        .thread_no(10)
        .step_delay(Duration::from_millis(500))
        .monitor(Monitor::Progress(100))
        .build();
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}

#[test]
pub fn rest_load_long() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(18_000))
        .thread_no(3)
        .step_delay(Duration::from_secs(1))
        .monitor(Monitor::Progress(10_000))
        .build();
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}
