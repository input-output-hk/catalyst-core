use crate::poem_types::event::EventId;
use crate::poem_types::registration::{Delegator, Voter, VotingKey};
use crate::service::common::responses::resp_2xx::OK;
use crate::service::common::responses::resp_4xx::NotFound;
use crate::service::common::responses::resp_5xx::ServerError;
use crate::{service::common::tags::ApiTags, state::State};
use poem::web::Data;
use poem_extensions::response;
use poem_extensions::UniResponse::{T200, T404, T500};
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};
use std::sync::Arc;

pub struct RegistrationApi;

#[OpenApi(prefix_path = "/registration", tag = "ApiTags::Registration")]
impl RegistrationApi {
    /// Voter's info
    ///
    /// Get voter's registration and voting power by their voting key.
    /// If the `event_id` query parameter is missing, then the "Latest" registration is returned.
    /// If the `with_delegators` query parameter is ommitted, then `delegator_addresses` field of `VoterInfo` type does not provided.
    ///
    #[oai(
        path = "/voter/:voting_key",
        method = "get",
        operation_id = "getVoterInfo"
    )]
    async fn get_voter_info(
        &self,
        pool: Data<&Arc<State>>,
        ///  A Voting Key.
        voting_key: Path<VotingKey>,
        /// The Event ID to return results for.
        event_id: Query<Option<EventId>>,
        /// Flag to include delegators list in the response.
        #[oai(default)]
        with_delegators: Query<bool>,
    ) -> response! {
           200: OK<Json<Voter>>,
           404: NotFound,
           500: ServerError,
       } {
        let voter = pool
            .event_db
            .get_voter(
                &event_id.0.map(Into::into),
                voting_key.0 .0,
                *with_delegators,
            )
            .await;
        match voter {
            Ok(voter) => T200(OK(Json(voter.into()))),
            Err(event_db::error::Error::NotFound(_)) => T404(NotFound),
            Err(err) => T500(ServerError::new(Some(err.to_string()))),
        }
    }

    /// Voter's delegations info
    ///
    /// Get voters delegation info by stake public key.
    /// If the `event_id` query parameter is missing, then the "Latest" registration is returned.
    #[oai(
        path = "/delegations/:stake_key",
        method = "get",
        operation_id = "getDelegatorInfo"
    )]
    async fn get_delegations(
        &self,
        pool: Data<&Arc<State>>,
        /// Stake public key.
        stake_public_key: Path<String>,
        /// The Event ID to return results for.
        event_id: Query<Option<EventId>>,
    ) -> response! {
           200: OK<Json<Delegator>>,
           404: NotFound,
           500: ServerError,
       } {
        let voter = pool
            .event_db
            .get_delegator(&event_id.0.map(Into::into), stake_public_key.0)
            .await;
        match voter {
            Ok(delegator) => T200(OK(Json(delegator.into()))),
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

        let resp = app
            .get(format!("/api/v1/registration/voter/{0}", "voting_key_1"))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "voter_info": {
                    "voting_power": 250,
                    "voting_group": "rep",
                    "delegations_power": 250,
                    "delegations_count": 2,
                    "voting_power_saturation": 0.625,
                },
                "as_at": "2022-03-31T12:00:00+00:00",
                "last_updated": "2022-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!(
                "/api/v1/registration/voter/{0}?with_delegators=true",
                "voting_key_1"
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "voter_info": {
                    "voting_power": 250,
                    "voting_group": "rep",
                    "delegations_power": 250,
                    "delegations_count": 2,
                    "voting_power_saturation": 0.625,
                    "delegator_addresses": ["stake_public_key_1", "stake_public_key_2"]
                },
                "as_at": "2022-03-31T12:00:00+00:00",
                "last_updated": "2022-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!(
                "/api/v1/registration/voter/{0}?event_id={1}",
                "voting_key_1", 1
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "voter_info": {
                    "voting_power": 250,
                    "voting_group": "rep",
                    "delegations_power": 250,
                    "delegations_count": 2,
                    "voting_power_saturation": 0.625,
                },
                "as_at": "2020-03-31T12:00:00+00:00",
                "last_updated": "2020-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!(
                "/api/v1/registration/voter/{0}?event_id={1}&with_delegators=true",
                "voting_key_1", 1
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "voter_info": {
                    "voting_power": 250,
                    "voting_group": "rep",
                    "delegations_power": 250,
                    "delegations_count": 2,
                    "voting_power_saturation": 0.625,
                    "delegator_addresses": ["stake_public_key_1", "stake_public_key_2"]
                },
                "as_at": "2020-03-31T12:00:00+00:00",
                "last_updated": "2020-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!("/api/v1/registration/voter/{0}", "voting_key"))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);

        let resp = app
            .get(format!("/api/v1/registration/voter/{0}", "voting_key"))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delegations_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = mk_test_app(state);

        let resp = app
            .get(format!(
                "/api/v1/registration/delegations/{0}",
                "stake_public_key_1"
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "delegations": [
                    {
                        "voting_key": "voting_key_1",
                        "group": "rep",
                        "weight": 1,
                        "value": 140,
                    },
                    {
                        "voting_key": "voting_key_2",
                        "group": "rep",
                        "weight": 1,
                        "value": 100,
                    },
                ],
                "reward_address": "addrrreward_address_1",
                "reward_payable": true,
                "raw_power": 240,
                "total_power": 1000,
                "as_at": "2022-03-31T12:00:00+00:00",
                "last_updated": "2022-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!(
                "/api/v1/registration/delegations/{0}?event_id={1}",
                "stake_public_key_1", 1
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::OK);
        resp.assert_json(serde_json::json!(
            {
                "delegations": [
                    {
                        "voting_key": "voting_key_1",
                        "group": "rep",
                        "weight": 1,
                        "value": 140,
                    },
                    {
                        "voting_key": "voting_key_2",
                        "group": "rep",
                        "weight": 1,
                        "value": 100,
                    },
                ],
                "reward_address": "addrrreward_address_1",
                "reward_payable": true,
                "raw_power": 240,
                "total_power": 1000,
                "as_at": "2020-03-31T12:00:00+00:00",
                "last_updated": "2020-03-31T12:00:00+00:00",
                "final": true
            }
        ))
        .await;

        let resp = app
            .get(format!(
                "/api/v1/registration/delegations/{0}",
                "stake_public_key"
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);

        let resp = app
            .get(format!(
                "/api/v1/registration/delegations/{0}?event_id={1}",
                "stake_public_key", 1
            ))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }
}
