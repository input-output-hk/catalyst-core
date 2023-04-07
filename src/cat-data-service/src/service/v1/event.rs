use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{routing::get, Router};
use event_db::types::event::EventSummary;
use std::sync::Arc;

pub fn event(state: Arc<State>) -> Router {
    Router::new().route(
        "/events",
        get({
            let state = state.clone();
            move || async { handle_result(events_exec(state).await).await }
        }),
    )
}

async fn events_exec(state: Arc<State>) -> Result<Vec<EventSummary>, Error> {
    tracing::debug!("events_exec");

    let events = state.event_db.get_events(None, None).await?;
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
    }
}
