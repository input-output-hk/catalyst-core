use crate::{
    service::{handle_result, v1::LimitOffset, Error},
    state::State,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use event_db::types::event::{objective::Objective, EventId};
use std::sync::Arc;

mod ballots;
mod proposal;
mod review_type;

pub fn objective(state: Arc<State>) -> Router {
    let proposal = proposal::proposal(state.clone());
    let review_type = review_type::review_type(state.clone());
    let ballots = ballots::ballots(state.clone());

    Router::new()
        .nest(
            "/objective/:objective",
            proposal.merge(review_type).merge(ballots),
        )
        .route(
            "/objectives",
            get(move |path, query| async {
                handle_result(objectives_exec(path, query, state).await).await
            }),
        )
}

async fn objectives_exec(
    Path(event): Path<EventId>,
    lim_ofs: Query<LimitOffset>,
    state: Arc<State>,
) -> Result<Vec<Objective>, Error> {
    tracing::debug!("objectives_query, event: {0}", event.0);

    let event = state
        .event_db
        .get_objectives(event, lim_ofs.limit, lim_ofs.offset)
        .await?;
    Ok(event)
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
        http::{Request, StatusCode},
    };
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn objectives_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objectives", 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            serde_json::Value::from_str(
                String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                    .unwrap()
                    .as_str()
            )
            .unwrap(),
            serde_json::json!(
                [
                    {
                        "id": 1,
                        "type": {
                            "id": "catalyst-simple",
                            "description": "A Simple choice"
                        },
                        "title": "title 1",
                        "description": "description 1",
                        "groups": [
                            {
                                "group": "direct",
                                "voting_token": "voting token 1"
                            },
                            {
                                "group": "rep",
                                "voting_token": "voting token 2"
                            }
                        ],
                        "reward": {
                            "currency": "ADA",
                            "value": 100
                        },
                        "supplemental": {
                            "url":"objective 1 url",
                            "sponsor": "objective 1 sponsor",
                            "video": "objective 1 video"
                        }
                    },
                    {
                        "id": 2,
                        "type": {
                            "id": "catalyst-native",
                            "description": "??"
                        },
                        "title": "title 2",
                        "description": "description 2",
                        "groups": [],
                    }
                ]
            )
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objectives?limit={1}", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            serde_json::Value::from_str(
                String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                    .unwrap()
                    .as_str()
            )
            .unwrap(),
            serde_json::json!(
                [
                    {
                        "id": 1,
                        "type": {
                            "id": "catalyst-simple",
                            "description": "A Simple choice"
                        },
                        "title": "title 1",
                        "description": "description 1",
                        "groups": [
                            {
                                "group": "direct",
                                "voting_token": "voting token 1"
                            },
                            {
                                "group": "rep",
                                "voting_token": "voting token 2"
                            }
                        ],
                        "reward": {
                            "currency": "ADA",
                            "value": 100
                        },
                        "supplemental": {
                            "url":"objective 1 url",
                            "sponsor": "objective 1 sponsor",
                            "video": "objective 1 video"
                        }
                    },
                ]
            )
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objectives?offset={1}", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            serde_json::Value::from_str(
                String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                    .unwrap()
                    .as_str()
            )
            .unwrap(),
            serde_json::json!(
                [
                    {
                        "id": 2,
                        "type": {
                            "id": "catalyst-native",
                            "description": "??"
                        },
                        "title": "title 2",
                        "description": "description 2",
                        "groups": [],
                    }
                ]
            )
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objectives?limit={1}&offset={2}",
                1, 1, 2
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&Vec::<Objective>::new()).unwrap()
        );
    }
}
