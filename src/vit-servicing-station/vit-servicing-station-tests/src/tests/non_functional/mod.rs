use crate::common::{clients::RestClient, startup::quick_start};
use assert_fs::TempDir;

use crate::common::clients::graphql::GraphqlClient;
use crate::common::data::Snapshot;
use jortestkit::load::{self, Configuration, Monitor, Request, RequestFailure};
use rand_core::{OsRng, RngCore};

#[derive(Clone, Debug)]
struct VitRestRequest {
    snapshot: Snapshot,
    rest_client: RestClient,
    graphql_client: GraphqlClient,
    random: OsRng,
}

impl VitRestRequest {
    pub fn new(
        snapshot: Snapshot,
        mut rest_client: RestClient,
        mut graphql_client: GraphqlClient,
    ) -> Self {
        rest_client.disable_log();
        graphql_client.disable_log();

        Self {
            snapshot,
            rest_client,
            graphql_client,
            random: OsRng,
        }
    }

    fn random_token(&self) -> String {
        let tokens = self.snapshot.tokens();
        let random_idx = self.radnom_usize() % tokens.len();
        tokens
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .get(random_idx)
            .unwrap()
            .clone()
    }

    fn radnom_usize(&self) -> usize {
        self.random.clone().next_u32() as usize
    }

    fn random_proposal_id(&self) -> i32 {
        let proposals = self.snapshot.proposals();
        let random_idx = self.radnom_usize() % proposals.len();
        proposals.get(random_idx).unwrap().internal_id.clone()
    }
    fn random_fund_id(&self) -> i32 {
        let funds = self.snapshot.funds();
        let random_idx = self.radnom_usize() % funds.len();
        funds.get(random_idx).unwrap().id.clone()
    }
}

impl Request for VitRestRequest {
    fn run(&self) -> Result<(), RequestFailure> {
        let random_token = self.random_token();

        let random_proposal_id = self.random_proposal_id();
        let random_fund_id = self.random_fund_id();

        let mut rest_client = self.rest_client.clone();
        rest_client.set_api_token(random_token.clone());

        let mut graphql_client = self.graphql_client.clone();
        graphql_client.set_api_token(random_token);

        match self.random.clone().next_u32() % 7 {
            0 => rest_client
                .health()
                .map_err(|e| RequestFailure::General(format!("Health: {}", e.to_string()))),
            1 => rest_client
                .proposals()
                .map(|_| ())
                .map_err(|e| RequestFailure::General(format!("Proposals: {}", e.to_string()))),
            2 => graphql_client
                .proposal_by_id(random_proposal_id as u32)
                .map(|_| ())
                .map_err(|e| {
                    RequestFailure::General(format!("GraohQL - Proposals by id: {}", e.to_string()))
                }),
            3 => graphql_client
                .fund_by_id(random_fund_id)
                .map(|_| ())
                .map_err(|e| {
                    RequestFailure::General(format!("GraphQL - Fund by id: {}", e.to_string()))
                }),
            4 => rest_client
                .proposal(&random_proposal_id.to_string())
                .map(|_| ())
                .map_err(|e| {
                    RequestFailure::General(format!("Proposals by id: {}", e.to_string()))
                }),
            5 => rest_client
                .fund(&random_fund_id.to_string())
                .map(|_| ())
                .map_err(|e| RequestFailure::General(format!("Funds by id: {}", e.to_string()))),
            6 => graphql_client
                .funds()
                .map(|_| ())
                .map_err(|e| RequestFailure::General(format!("Funds: {}", e.to_string()))),
            _ => unreachable!(),
        }
    }
}

#[test]
pub fn rest_load() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();
    let graphql_client = server.graphql_client();

    let request = VitRestRequest::new(snapshot, rest_client, graphql_client);
    let config = Configuration::duration(
        10,
        std::time::Duration::from_secs(40),
        500,
        Monitor::Progress(100),
    );
    load::start(request, config, "Vit station service rest");
}
