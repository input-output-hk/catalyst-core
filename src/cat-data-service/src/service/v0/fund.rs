use crate::{
    service::{handle_result, Error},
    state::State,
    types::SerdeType,
};
use axum::{routing::get, Router};
use event_db::types::vit_ss::fund::FundWithNext;
use std::sync::Arc;

pub fn fund(state: Arc<State>) -> Router {
    Router::new().route(
        "/fund",
        get(|| async { handle_result(fund_exec(state).await) }),
    )
}

async fn fund_exec(state: Arc<State>) -> Result<SerdeType<FundWithNext>, Error> {
    tracing::debug!("fund_query",);

    let fund = state.event_db.get_fund().await?.into();
    Ok(fund)
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
    async fn fund_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri("/api/v0/fund".to_string())
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!(
                {
                    "id": 10,
                    "fund_name": "Fund 10",
                    "fund_goal": "Catalyst Dev Environment - Fund 10",
                    "voting_power_threshold": 450000000,
                    "fund_start_time": "2023-06-16T19:56:00+00:00",
                    "fund_end_time": "2023-09-18T00:00:00+00:00",
                    "next_fund_start_time": "1970-01-01T00:00:00+00:00",
                    "registration_snapshot_time": "2023-08-18T21:00:00+00:00",
                    "next_registration_snapshot_time": "1970-01-01T00:00:00+00:00",
                    "chain_vote_plans": [],
                    "challenges": [],
                    "insight_sharing_start": "2023-06-22T00:00:00+00:00",
                    "proposal_submission_start": "2023-06-22T00:00:00+00:00",
                    "refine_proposals_start": "2023-06-22T00:00:00+00:00",
                    "finalize_proposals_start": "2023-07-13T00:00:00+00:00",
                    "proposal_assessment_start": "2023-07-20T00:00:00+00:00",
                    "assessment_qa_start": "2023-08-10T00:00:00+00:00",
                    "snapshot_start": "2023-08-23T22:00:00+00:00",
                    "voting_start": "2023-08-31T11:00:00+00:00",
                    "voting_end": "2023-09-14T11:00:00+00:00",
                    "tallying_end": "2023-09-18T11:00:00+00:00",
                    "goals": [],
                    "groups": [],
                    "survey_url": "",
                    "results_url": "",
                }
            ),
        );
    }
}
