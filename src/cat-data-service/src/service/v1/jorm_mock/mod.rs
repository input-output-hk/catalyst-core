use crate::{
    service::{handle_result, Error},
    state::State,
    types::jorm_mock::{AccountId, AccountVote, Fragments, FragmentsProcessingSummary},
};
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

pub fn jorm_mock(state: Arc<State>) -> Router {
    Router::new()
        .route("/fragments", {
            let state = state.clone();
            post(move |body| async { handle_result(fragments_exec(body, state).await) })
        })
        .route(
            "/votes/plan/account-votes/:account_id",
            get(move |path| async { handle_result(account_votes_exec(path, state).await) }),
        )
}

async fn fragments_exec(
    Json(fragments_query): Json<Fragments>,
    state: Arc<State>,
) -> Result<FragmentsProcessingSummary, Error> {
    tracing::debug!("fragments query",);
    let mut jorm = state.jorm.lock().unwrap();
    let res = jorm.accept_fragments(fragments_query.fragments);
    Ok(res)
}

async fn account_votes_exec(
    Path(account_id): Path<AccountId>,
    state: Arc<State>,
) -> Result<Vec<AccountVote>, Error> {
    tracing::debug!(
        "account votes query, account_id: {}",
        account_id.to_string()
    );
    let jorm = state.jorm.lock().unwrap();
    Ok(jorm.get_account_votes(&account_id))
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
        http::{header, Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn fragments_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/fragments".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::json!({
                    "fail_fast": false,
                    "fragments": []
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!({
                "accepted": [],
                "rejected": []
            })
        ));
    }

    #[tokio::test]
    async fn account_votes_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri(format!(
                "/api/v1/votes/plan/account-votes/{0}",
                "0000000000000000000000000000000000000000"
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_data_json_check(
            response.into_body().data().await.unwrap().unwrap().to_vec(),
            serde_json::json!([])
        ));
    }
}
