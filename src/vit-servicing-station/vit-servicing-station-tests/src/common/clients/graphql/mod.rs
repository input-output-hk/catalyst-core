use super::{RestClient, RestError};
use askama::Template;
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::Proposal;

pub mod templates;

use serde_json::Value;

pub struct GraphqlClient {
    rest_client: RestClient,
}

impl GraphqlClient {
    pub fn new(address: String) -> Self {
        Self {
            rest_client: RestClient::new(address),
        }
    }

    pub fn proposal_by_id(&self, id: u32) -> Result<Proposal, GraphQlClientError> {
        let proposal = templates::ProposalById { id };
        let data = proposal.render()?.replace("\r\n", "").replace("\n", "");
        let query_result = self.run_query(&data)?;
        serde_json::from_value(query_result["data"]["proposal"].clone())
            .map_err(GraphQlClientError::CannotDeserialize)
    }

    pub fn run_query(&self, data: &str) -> Result<Value, GraphQlClientError> {
        self.rest_client
            .post("graphql", data.replace("\r\n", " ").replace("\n", " "))
            .map_err(GraphQlClientError::RestError)
    }

    pub fn set_api_token(&mut self, token: String) {
        self.rest_client.set_api_token(token);
    }
}

#[derive(Debug, Error)]
pub enum GraphQlClientError {
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    RequestError(#[from] reqwest::Error),
    #[error("could not serializa template")]
    TemplateError(#[from] askama_shared::Error),
    #[error("rest error")]
    RestError(#[from] RestError),
}
