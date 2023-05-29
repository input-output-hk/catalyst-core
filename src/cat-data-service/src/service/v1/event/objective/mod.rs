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

mod proposal;
mod review_type;

pub fn objective(state: Arc<State>) -> Router {
    let proposal = proposal::proposal(state.clone());
    let review_type = review_type::review_type(state.clone());

    Router::new()
        .nest("/objective/:objective", proposal.merge(review_type))
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
    use event_db::types::{
        event::objective::{
            ObjectiveDetails, ObjectiveId, ObjectiveSummary, ObjectiveSupplementalData,
            ObjectiveType, RewardDefintion, VoterGroup,
        },
        registration::VoterGroupId,
    };
    use serde_json::json;
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
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![
                Objective {
                    summary: ObjectiveSummary {
                        id: ObjectiveId(1),
                        objective_type: ObjectiveType {
                            id: "catalyst-simple".to_string(),
                            description: "A Simple choice".to_string()
                        },
                        title: "title 1".to_string(),
                        description: "description 1".to_string(),
                    },
                    details: ObjectiveDetails {
                        groups: vec![
                            VoterGroup {
                                group: Some(VoterGroupId("direct".to_string())),
                                voting_token: Some("voting token 1".to_string()),
                            },
                            VoterGroup {
                                group: Some(VoterGroupId("rep".to_string())),
                                voting_token: Some("voting token 2".to_string()),
                            }
                        ],
                        reward: Some(RewardDefintion {
                            currency: "ADA".to_string(),
                            value: 100
                        }),
                        supplemental: Some(ObjectiveSupplementalData(json!(
                            {
                                "url":"objective 1 url",
                                "sponsor": "objective 1 sponsor",
                                "video": "objective 1 video"
                            }
                        ))),
                    }
                },
                Objective {
                    summary: ObjectiveSummary {
                        id: ObjectiveId(2),
                        objective_type: ObjectiveType {
                            id: "catalyst-native".to_string(),
                            description: "??".to_string()
                        },
                        title: "title 2".to_string(),
                        description: "description 2".to_string(),
                    },
                    details: ObjectiveDetails {
                        groups: Vec::new(),
                        reward: None,
                        supplemental: None,
                    }
                }
            ])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objectives?limit={1}", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![Objective {
                summary: ObjectiveSummary {
                    id: ObjectiveId(1),
                    objective_type: ObjectiveType {
                        id: "catalyst-simple".to_string(),
                        description: "A Simple choice".to_string()
                    },
                    title: "title 1".to_string(),
                    description: "description 1".to_string(),
                },
                details: ObjectiveDetails {
                    groups: vec![
                        VoterGroup {
                            group: Some(VoterGroupId("direct".to_string())),
                            voting_token: Some("voting token 1".to_string()),
                        },
                        VoterGroup {
                            group: Some(VoterGroupId("rep".to_string())),
                            voting_token: Some("voting token 2".to_string()),
                        }
                    ],
                    reward: Some(RewardDefintion {
                        currency: "ADA".to_string(),
                        value: 100
                    }),
                    supplemental: Some(ObjectiveSupplementalData(json!(
                        {
                            "url":"objective 1 url",
                            "sponsor": "objective 1 sponsor",
                            "video": "objective 1 video"
                        }
                    ))),
                }
            },])
            .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objectives?offset={1}", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap(),
            serde_json::to_string(&vec![Objective {
                summary: ObjectiveSummary {
                    id: ObjectiveId(2),
                    objective_type: ObjectiveType {
                        id: "catalyst-native".to_string(),
                        description: "??".to_string()
                    },
                    title: "title 2".to_string(),
                    description: "description 2".to_string(),
                },
                details: ObjectiveDetails {
                    groups: Vec::new(),
                    reward: None,
                    supplemental: None,
                }
            }])
            .unwrap()
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
