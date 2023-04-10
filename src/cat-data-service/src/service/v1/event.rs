use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use event_db::types::event::{Event, EventId, EventSummary};
use serde::Deserialize;
use std::sync::Arc;

pub fn event(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/event/:event",
            get({
                let state = state.clone();
                move |path| async { handle_result(event_exec(path, state).await).await }
            }),
        )
        .route(
            "/events",
            get({
                let state = state.clone();
                move |query| async { handle_result(events_exec(query, state).await).await }
            }),
        )
}

async fn event_exec(Path(event): Path<EventId>, state: Arc<State>) -> Result<Event, Error> {
    tracing::debug!("event_exec, event: {0}", event.0);

    let event = state.event_db.get_event(event).await?;
    Ok(event)
}

#[derive(Deserialize)]
struct EventsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn events_exec(
    events_query: Query<EventsQuery>,
    state: Arc<State>,
) -> Result<Vec<EventSummary>, Error> {
    tracing::debug!(
        "events_exec, limit: {0:?}, offset: {1:?}",
        events_query.limit,
        events_query.offset
    );

    let events = state
        .event_db
        .get_events(events_query.limit, events_query.offset)
        .await?;
    Ok(events)
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
    use event_db::types::event::EventId;
    use tower::ServiceExt;

    #[tokio::test]
    async fn event_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}", 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&EventSummary {
                id: EventId(1),
                name: "Test Fund 1".to_string(),
                starts: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                ends: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
                reg_checked: None,
            },)
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}", 10))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn events_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/events"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![
                EventSummary {
                    id: EventId(1),
                    name: "Test Fund 1".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(2),
                    name: "Test Fund 2".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(3),
                    name: "Test Fund 3".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                }
            ])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/events?offset={0}", 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![
                EventSummary {
                    id: EventId(2),
                    name: "Test Fund 2".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(3),
                    name: "Test Fund 3".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                }
            ])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/events?limit={0}", 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![EventSummary {
                id: EventId(1),
                name: "Test Fund 1".to_string(),
                starts: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                ends: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
                reg_checked: None,
            },])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/events?limit={0}&offset={1}", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![EventSummary {
                id: EventId(2),
                name: "Test Fund 2".to_string(),
                starts: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                ends: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
                reg_checked: None,
            },])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/events?offset={0}", 10))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Vec::<EventSummary>::new()).unwrap()
        );
    }
}
