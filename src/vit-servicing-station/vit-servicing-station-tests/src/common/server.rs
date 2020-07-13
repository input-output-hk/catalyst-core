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

    pub fn settings(&self) -> ServiceSettings {
        self.settings.clone()
    }

    pub fn rest_client_with_token(&self, token: &str) -> RestClient {
        let mut rest_client = self.rest_client();
        rest_client.set_api_token(token.to_string());
        rest_client
    }

    pub fn graphql_client(&self) -> GraphqlClient {
        GraphqlClient::new(self.settings.address.to_string())
    }

    pub fn graphql_client_with_token(&self, token: &str) -> GraphqlClient {
        let mut graphql_client = self.graphql_client();
        graphql_client.set_api_token(token.to_string());
        graphql_client
    }

    pub fn is_token_valid(&self, token: &str) -> bool {
        self.is_up(token)
    }

    pub fn is_up(&self, token: &str) -> bool {
        self.rest_client_with_token(token).health().is_ok()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        self.process.wait().unwrap();
    }
}
