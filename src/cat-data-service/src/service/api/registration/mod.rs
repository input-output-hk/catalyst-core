use crate::poem_types::registration::{Voter, VotingKey};
use crate::service::common::responses::resp_2xx::OK;
use crate::service::common::responses::resp_4xx::NotFound;
use crate::service::common::responses::resp_5xx::ServerError;
use crate::{service::common::tags::ApiTags, state::State};
use poem::web::Data;
use poem_extensions::response;
use poem_extensions::UniResponse::{T200, T404, T500};
use poem_openapi::{param::Path, payload::Json, OpenApi};
use std::sync::Arc;

pub struct RegistrationApi;

#[OpenApi(prefix_path = "/registration", tag = "ApiTags::Registration")]
impl RegistrationApi {
    #[oai(
        path = "/voter/:voting_key",
        method = "get",
        operation_id = "getVoterInfo"
    )]
    /// Voter's info
    ///
    /// Get voter's registration and voting power by their voting key.
    /// If the `event_id` query parameter is omitted, then the latest voting power is retrieved.
    /// If the `with_delegators` query parameter is ommitted, then `delegator_addresses` field of `VoterInfo` type does not provided.
    ///
    async fn get_voter_info(
        &self,
        pool: Data<&Arc<State>>,
        voting_key: Path<VotingKey>,
    ) -> response! {
           200: OK<Json<Voter>>,
           404: NotFound,
           500: ServerError,
       } {
        let voter = pool.event_db.get_voter(&None, voting_key.0 .0, false).await;
        match voter {
            Ok(voter) => T200(OK(Json(voter.into()))),
            Err(event_db::error::Error::NotFound(_)) => T404(NotFound),
            Err(err) => T500(ServerError::new(Some(err.to_string()))),
        }
    }
}

/// Need to setup and run a test event db instance
/// To do it you can use the following commands:
/// Prepare docker images
/// ```
/// earthly ./containers/event-db-migrations+docker --data=test
/// ```
/// Run event-db container
/// ```
/// docker-compose -f src/event-db/docker-compose.yml up migrations
/// ```
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use crate::{service::poem_service::tests::mk_test_app, state::State};
    use poem::http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn voter_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = mk_test_app(state);

        let resp = app.get("/api/health/started").send().await;
        resp.assert_status(StatusCode::NO_CONTENT);

        let resp = app.get("/api/health/ready").send().await;
        resp.assert_status(StatusCode::NO_CONTENT);

        let resp = app.get("/api/health/live").send().await;
        resp.assert_status(StatusCode::NO_CONTENT);
    }
}
