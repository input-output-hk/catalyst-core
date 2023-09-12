use crate::poem_types::registration::{Voter, VotingKey};
use crate::service::common::responses::resp_2xx::OK;
use crate::service::common::responses::resp_4xx::NotFound;
use crate::service::common::responses::resp_5xx::ServerError;
use crate::state::State;
use poem_extensions::response;
use poem_extensions::UniResponse::{T200, T404, T500};
use poem_openapi::payload::Json;

pub type AllResponses = response! {
    200: OK<Json<Voter>>,
    404: NotFound,
    500: ServerError,
};

pub async fn endpoint(state: &State, voting_key: VotingKey) -> AllResponses {
    let voter = state.event_db.get_voter(&None, voting_key.0, false).await;
    match voter {
        Ok(voter) => T200(OK(Json(voter.into()))),
        Err(event_db::error::Error::NotFound(_)) => T404(NotFound),
        Err(err) => T500(ServerError::new(Some(err.to_string()))),
    }
}
