use super::clients::{GraphqlClient, RestClient};
use std::process::Child;
use vit_servicing_station_lib::server::settings::ServiceSettings;

pub struct Server {
    process: Child,
    settings: ServiceSettings,
}

impl Server {
    pub fn new(process: Child, settings: ServiceSettings) -> Self {
        Self { process, settings }
    }

    pub fn rest_client(&self) -> RestClient {
        RestClient::new(self.settings.address.to_string())
    }

    pub fn rest_client_with_token(&self, hash: String) -> RestClient {
        let mut rest_client = self.rest_client();
        rest_client.set_api_token(hash);
        rest_client
    }

    pub fn graphql_client(&self) -> GraphqlClient {
        GraphqlClient::new(self.settings.address.to_string())
    }

    pub fn graphql_client_with_token(&self, hash: String) -> GraphqlClient {
        let mut graphql_client = self.graphql_client();
        graphql_client.set_api_token(hash);
        graphql_client
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        self.process.wait().unwrap();
    }
}
