use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Query, routing::post, Json, Router};
use event_db::types::search::{SearchQuery, SearchResult};
use serde::Deserialize;
use std::sync::Arc;

pub fn search(state: Arc<State>) -> Router {
    Router::new().route(
        "/search",
        post(move |query, body| async {
            handle_result(search_exec(query, body, state).await).await
        }),
    )
}

/// Cannot use serde flattening, look this issue <https://github.com/nox/serde_urlencoded/issues/33>
#[derive(Deserialize)]
struct SearchParam {
    #[serde(default)]
    total: bool,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn search_exec(
    search_param: Query<SearchParam>,
    Json(search_query): Json<SearchQuery>,
    state: Arc<State>,
) -> Result<SearchResult, Error> {
    tracing::debug!("search_query",);

    let res = state
        .event_db
        .search(
            search_query,
            search_param.total,
            search_param.limit,
            search_param.offset,
        )
        .await?;
    Ok(res)
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
    use crate::service::app;
    use axum::{
        body::{Body, HttpBody},
        http::{header, Method, Request, StatusCode},
    };
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
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
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 4,
                results: Some(ValueResults::Events(vec![
                    EventSummary {
                        id: EventId(1),
                        name: "Test Fund 1".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(2),
                        name: "Test Fund 2".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(3),
                        name: "Test Fund 3".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(4),
                        name: "Test Fund 4".to_string(),
                        starts: None,
                        ends: None,
                        reg_checked: None,
                        is_final: false,
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search?total=true".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 4,
                results: None
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 4,
                results: Some(ValueResults::Events(vec![
                    EventSummary {
                        id: EventId(4),
                        name: "Test Fund 4".to_string(),
                        starts: None,
                        ends: None,
                        reg_checked: None,
                        is_final: false,
                    },
                    EventSummary {
                        id: EventId(3),
                        name: "Test Fund 3".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(2),
                        name: "Test Fund 2".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(1),
                        name: "Test Fund 1".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?limit={0}", 2))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 2,
                results: Some(ValueResults::Events(vec![
                    EventSummary {
                        id: EventId(4),
                        name: "Test Fund 4".to_string(),
                        starts: None,
                        ends: None,
                        reg_checked: None,
                        is_final: false,
                    },
                    EventSummary {
                        id: EventId(3),
                        name: "Test Fund 3".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?offset={0}", 2))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 2,
                results: Some(ValueResults::Events(vec![
                    EventSummary {
                        id: EventId(2),
                        name: "Test Fund 2".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2021, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                    EventSummary {
                        id: EventId(1),
                        name: "Test Fund 1".to_string(),
                        starts: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        ends: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        reg_checked: Some(DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                                NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                            ),
                            Utc
                        )),
                        is_final: true,
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?limit={0}&offset={1}", 1, 1))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "events",
                    "filter": [{
                        "column": "description",
                        "search": "Fund"
                    }],
                    "order_by": [{
                        "column": "description",
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
                results: Some(ValueResults::Events(vec![EventSummary {
                    id: EventId(3),
                    name: "Test Fund 3".to_string(),
                    starts: Some(DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    )),
                    ends: Some(DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    )),
                    reg_checked: Some(DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    )),
                    is_final: true,
                },]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
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
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "description",
                        "search": "description"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 2,
                results: Some(ValueResults::Objectives(vec![
                    ObjectiveSummary {
                        id: ObjectiveId(1),
                        objective_type: ObjectiveType {
                            id: "catalyst-simple".to_string(),
                            description: "A Simple choice".to_string()
                        },
                        title: "title 1".to_string(),
                        description: "description 1".to_string(),
                    },
                    ObjectiveSummary {
                        id: ObjectiveId(2),
                        objective_type: ObjectiveType {
                            id: "catalyst-native".to_string(),
                            description: "??".to_string()
                        },
                        title: "title 2".to_string(),
                        description: "description 2".to_string(),
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search?total=true".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "description",
                        "search": "description"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 2,
                results: None
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "description",
                        "search": "description"
                    }],
                    "order_by": [{
                        "column": "description",
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
                total: 2,
                results: Some(ValueResults::Objectives(vec![
                    ObjectiveSummary {
                        id: ObjectiveId(2),
                        objective_type: ObjectiveType {
                            id: "catalyst-native".to_string(),
                            description: "??".to_string()
                        },
                        title: "title 2".to_string(),
                        description: "description 2".to_string(),
                    },
                    ObjectiveSummary {
                        id: ObjectiveId(1),
                        objective_type: ObjectiveType {
                            id: "catalyst-simple".to_string(),
                            description: "A Simple choice".to_string()
                        },
                        title: "title 1".to_string(),
                        description: "description 1".to_string(),
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?limit={0}", 1))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "description",
                        "search": "description"
                    }],
                    "order_by": [{
                        "column": "description",
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
                results: Some(ValueResults::Objectives(vec![ObjectiveSummary {
                    id: ObjectiveId(2),
                    objective_type: ObjectiveType {
                        id: "catalyst-native".to_string(),
                        description: "??".to_string()
                    },
                    title: "title 2".to_string(),
                    description: "description 2".to_string(),
                },]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?offset={0}", 1))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "objectives",
                    "filter": [{
                        "column": "description",
                        "search": "description"
                    }],
                    "order_by": [{
                        "column": "description",
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
                results: Some(ValueResults::Objectives(vec![ObjectiveSummary {
                    id: ObjectiveId(1),
                    objective_type: ObjectiveType {
                        id: "catalyst-simple".to_string(),
                        description: "A Simple choice".to_string()
                    },
                    title: "title 1".to_string(),
                    description: "description 1".to_string(),
                },]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
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
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
                    }],
                    "order_by": [{
                        "column": "title",
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
                total: 3,
                results: Some(ValueResults::Proposals(vec![
                    ProposalSummary {
                        id: 1,
                        title: String::from("title 1"),
                        summary: String::from("summary 1")
                    },
                    ProposalSummary {
                        id: 2,
                        title: String::from("title 2"),
                        summary: String::from("summary 2")
                    },
                    ProposalSummary {
                        id: 3,
                        title: String::from("title 3"),
                        summary: String::from("summary 3")
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search?total=true".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
                    }],
                    "order_by": [{
                        "column": "title",
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
                total: 3,
                results: None
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
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
                total: 3,
                results: Some(ValueResults::Proposals(vec![
                    ProposalSummary {
                        id: 3,
                        title: String::from("title 3"),
                        summary: String::from("summary 3")
                    },
                    ProposalSummary {
                        id: 2,
                        title: String::from("title 2"),
                        summary: String::from("summary 2")
                    },
                    ProposalSummary {
                        id: 1,
                        title: String::from("title 1"),
                        summary: String::from("summary 1")
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?limit={0}", 2))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
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
                total: 2,
                results: Some(ValueResults::Proposals(vec![
                    ProposalSummary {
                        id: 3,
                        title: String::from("title 3"),
                        summary: String::from("summary 3")
                    },
                    ProposalSummary {
                        id: 2,
                        title: String::from("title 2"),
                        summary: String::from("summary 2")
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?offset={0}", 1))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
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
                total: 2,
                results: Some(ValueResults::Proposals(vec![
                    ProposalSummary {
                        id: 2,
                        title: String::from("title 2"),
                        summary: String::from("summary 2")
                    },
                    ProposalSummary {
                        id: 1,
                        title: String::from("title 1"),
                        summary: String::from("summary 1")
                    },
                ]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/v1/search?limit={0}&offset={1}", 1, 1))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "title",
                        "search": "title"
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
                results: Some(ValueResults::Proposals(vec![ProposalSummary {
                    id: 2,
                    title: String::from("title 2"),
                    summary: String::from("summary 2")
                },]))
            })
            .unwrap()
        );

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/search".to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "table": "proposals",
                    "filter": [{
                        "column": "description",
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
