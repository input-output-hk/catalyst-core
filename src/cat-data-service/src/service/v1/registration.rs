use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use event_db::types::{
    event::EventId,
    registration::{Delegator, Voter},
};
use serde::Deserialize;
use std::sync::Arc;

pub fn registration(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/registration/voter/:voting_key",
            get({
                let state = state.clone();
                move |path, query| async { handle_result(voter_exec(path, query, state).await) }
            }),
        )
        .route(
            "/registration/delegations/:stake_public_key",
            get({
                move |path, query| async {
                    handle_result(delegations_exec(path, query, state).await)
                }
            }),
        )
}

#[derive(Deserialize)]
struct VotersQuery {
    event_id: Option<EventId>,
    with_delegators: Option<bool>,
}

async fn voter_exec(
    Path(voting_key): Path<String>,
    voters_query: Query<VotersQuery>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!(
        "voter_query: voting_key: {0}, event_id: {1:?}",
        voting_key,
        voters_query.event_id
    );

    let voter = state
        .event_db
        .get_voter(
            &voters_query.event_id,
            voting_key,
            voters_query.with_delegators.unwrap_or(false),
        )
        .await?;
    Ok(voter)
}

#[derive(Deserialize)]
struct DelegationsQuery {
    event_id: Option<EventId>,
}

async fn delegations_exec(
    Path(stake_public_key): Path<String>,
    delegations_query: Query<DelegationsQuery>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!(
        "delegator_query: stake_public_key: {0}, eid: {1:?}",
        stake_public_key,
        delegations_query.event_id
    );

    let delegator = state
        .event_db
        .get_delegator(&delegations_query.event_id, stake_public_key)
        .await?;
    Ok(delegator)
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
    use super::*;
    use crate::service::{app, tests::body_data_json_check};
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn voter_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/registration/voter/{0}", "voting_key_1"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}?with_delegators=true",
                "voting_key_1"
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}?event_id={1}",
                "voting_key_1", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}?event_id={1}&with_delegators=true",
                "voting_key_1", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!("/api/v1/registration/voter/{0}", "voting_key"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}?event_id={1}",
                "voting_key", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delegations_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}",
                "stake_public_key_1"
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}?event_id={1}",
                "stake_public_key_1", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
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
            )
        ));

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}",
                "stake_public_key"
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}?event_id={1}",
                "stake_public_key", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
