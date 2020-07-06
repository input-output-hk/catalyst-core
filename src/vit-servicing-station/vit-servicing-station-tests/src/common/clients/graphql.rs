use askama::Template;
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::Proposal;

use super::{RestClient, RestError};

#[derive(Template)]
#[template(path = "proposal_by_id.graphql.txt")]
struct ProposalById {
    id: u32,
}

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
        let proposal = ProposalById { id };
        let data = proposal.render()?;
        println!("Request: {}", data);
        let query_result = self.rest_client.post("graphql", data.replace("\r\n", ""))?;
        let proposal = query_result["data"]["proposal"].clone();
        serde_json::from_value(proposal).map_err(GraphQlClientError::CannotDeserialize)
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
