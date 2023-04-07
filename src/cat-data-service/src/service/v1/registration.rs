use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use event_db::types::{
    event::EventId,
    registration::{Delegator, Voter},
};
use std::sync::Arc;

pub fn registration(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/registration/voter/:voting_key",
            get({
                let state = state.clone();
                move |path| async {
                    handle_result(voter_by_latest_event_exec(path, state).await).await
                }
            }),
        )
        .route(
            "/registration/voter/:voting_key/:event",
            get({
                let state = state.clone();
                move |path| async { handle_result(voter_by_event_exec(path, state).await).await }
            }),
        )
        .route(
            "/registration/delegations/:stake_public_key",
            get({
                let state = state.clone();
                move |path| async {
                    handle_result(delegations_by_latest_event_exec(path, state).await).await
                }
            }),
        )
        .route(
            "/registration/delegations/:stake_public_key/:event",
            get(move |path| async {
                handle_result(delegations_by_event_exec(path, state).await).await
            }),
        )
}

async fn voter_by_latest_event_exec(
    Path(voting_key): Path<String>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!("voter_by_latest_event_exec: voting_key: {0}", voting_key);

    let voter = state.event_db.get_voter(&None, voting_key).await?;
    Ok(voter)
}

async fn voter_by_event_exec(
    Path((voting_key, event)): Path<(String, EventId)>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!(
        "voter_by_event_exec: voting_key: {0}, event: {1}",
        voting_key,
        event.0
    );

    let voter = state.event_db.get_voter(&Some(event), voting_key).await?;
    Ok(voter)
}

async fn delegations_by_latest_event_exec(
    Path(stake_public_key): Path<String>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!("delegator_exec: stake_public_key: {0}", stake_public_key);

    let delegator = state
        .event_db
        .get_delegator(&None, stake_public_key)
        .await?;
    Ok(delegator)
}

async fn delegations_by_event_exec(
    Path((stake_public_key, event)): Path<(String, EventId)>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!(
        "delegator_exec: stake_public_key: {0}, event: {1}",
        stake_public_key,
        event.0
    );

    let delegator = state
        .event_db
        .get_delegator(&Some(event), stake_public_key)
        .await?;
    Ok(delegator)
}

/// Need to setup and run a test event db instance
/// To do it you can use `cargo make local-event-db-setup`
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::app;
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use event_db::types::registration::{Delegation, VoterInfo};
    use tower::ServiceExt;

    #[tokio::test]
    async fn voter_by_latest_event_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/registration/voter/{0}", "voting_key_1"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Voter {
                voter_info: VoterInfo {
                    voting_power: 250,
                    voting_group: "rep".to_string(),
                    delegations_power: 250,
                    delegations_count: 2,
                    voting_power_saturation: 0.625,
                },
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
            })
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/registration/voter/{0}", "voting_key"))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn voter_by_event_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}/{1}",
                "voting_key_1", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Voter {
                voter_info: VoterInfo {
                    voting_power: 250,
                    voting_group: "rep".to_string(),
                    delegations_power: 250,
                    delegations_count: 2,
                    voting_power_saturation: 0.625,
                },
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
            })
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/voter/{0}/{1}",
                "voting_key", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delegations_by_latest_event_test() {
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
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Delegator {
                delegations: vec![
                    Delegation {
                        voting_key: "voting_key_1".to_string(),
                        group: "rep".to_string(),
                        weight: 1,
                        value: 140
                    },
                    Delegation {
                        voting_key: "voting_key_2".to_string(),
                        group: "rep".to_string(),
                        weight: 1,
                        value: 100
                    }
                ],
                raw_power: 240,
                total_power: 1000,
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true
            })
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}",
                "stake_public_key"
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delegations_by_event_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}/{1}",
                "stake_public_key_1", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Delegator {
                delegations: vec![
                    Delegation {
                        voting_key: "voting_key_1".to_string(),
                        group: "rep".to_string(),
                        weight: 1,
                        value: 140
                    },
                    Delegation {
                        voting_key: "voting_key_2".to_string(),
                        group: "rep".to_string(),
                        weight: 1,
                        value: 100
                    }
                ],
                raw_power: 240,
                total_power: 1000,
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true
            })
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/registration/delegations/{0}/{1}",
                "stake_public_key", 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
