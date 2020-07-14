use crate::common::{clients::RestClient, startup::quick_start};
use assert_fs::TempDir;

use jortestkit::load::{self, Configuration, Monitor, Request, RequestFailure};
use rand_core::{OsRng, RngCore};

#[derive(Clone, Debug)]
struct VitRestRequest {
    rest_client: RestClient,
    random: OsRng,
}

impl From<RestClient> for VitRestRequest {
    fn from(rest_client: RestClient) -> Self {
        Self {
            rest_client,
            random: OsRng,
        }
    }
}

impl Request for VitRestRequest {
    fn run(&self) -> Result<(), RequestFailure> {
        match self.random.clone().next_u32() % 2 {
            0 => self
                .rest_client
                .health()
                .map_err(|e| RequestFailure::General(format!("Health: {}", e.to_string()))),
            1 => self
                .rest_client
                .proposals()
                .map(|_| ())
                .map_err(|e| RequestFailure::General(format!("Proposals: {}", e.to_string()))),
            _ => unreachable!(),
        }
    }
}

#[test]
pub fn rest_load() {
    let temp_dir = TempDir::new().unwrap();
    let (server, token) = quick_start(&temp_dir).unwrap();
    let mut rest_client = server.rest_client_with_token(&token);
    rest_client.disable_log();
    let request: VitRestRequest = rest_client.into();
    let config = Configuration::duration(
        10,
        std::time::Duration::from_secs(40),
        500,
        Monitor::Progress(100),
    );
    load::start(request, config, "Vit station service rest");
}
