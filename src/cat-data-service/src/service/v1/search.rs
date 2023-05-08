use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{routing::post, Json, Router};
use event_db::types::search::{SearchQuery, SearchResult};
use std::sync::Arc;

pub fn search(state: Arc<State>) -> Router {
    Router::new().route(
        "/search",
        post(move |body| async { handle_result(search_exec(body, state).await).await }),
    )
}

async fn search_exec(
    Json(search_query): Json<SearchQuery>,
    state: Arc<State>,
) -> Result<SearchResult, Error> {
    tracing::debug!("search_query",);

    let res = state.event_db.search(search_query, None, None).await?;
    Ok(res)
}

/// Need to setup and run a test event db instance
/// To do it you can use `cargo make local-event-db-test`
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
        http::{header, Method, Request, StatusCode},
    };
    use event_db::types::{
        event::{
            objective::{ObjectiveId, ObjectiveSummary, ObjectiveType},
            proposal::ProposalSummary,
            EventId, EventSummary,
        },
        search::ValueResults,
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn search_events_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "desc",
                        "search": "Fund 4"
                    }],
                    "order_by": [{
                        "column": "desc",
                        "descending": true,
                    }]
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&SearchResult {
                total: 1,
                results: ValueResults::Events(vec![EventSummary {
                    id: EventId(4),
                    name: "Test Fund 4".to_string(),
                    starts: None,
                    ends: None,
                    reg_checked: None,
                    is_final: false,
                }])
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "funds",
                        "search": "Fund 4"
                    }],
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn search_objectives_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "desc",
                        "search": "description 1"
                    }],
                    "order_by": [{
                        "column": "desc",
                        "descending": true,
                    }]
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&SearchResult {
                total: 1,
                results: ValueResults::Objectives(vec![ObjectiveSummary {
                    id: ObjectiveId(1),
                    objective_type: ObjectiveType {
                        id: "catalyst-simple".to_string(),
                        description: "A Simple choice".to_string()
                    },
                    title: "title 1".to_string(),
                }])
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "funds",
                        "search": "description 1"
                    }],
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn search_proposals_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title 1"
                    }],
                    "order_by": [{
                        "column": "title",
                        "descending": true,
                    }]
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&SearchResult {
                total: 1,
                results: ValueResults::Proposals(vec![ProposalSummary {
                    id: 1,
                    title: String::from("title 1"),
                    summary: String::from("summary 1")
                },])
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "desc",
                        "search": "description 1"
                    }],
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
