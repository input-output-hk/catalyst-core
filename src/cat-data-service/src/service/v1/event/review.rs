use crate::{
    service::{handle_result, v1::LimitOffset, Error},
    state::State,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use event_db::types::event::{
    objective::ObjectiveId,
    proposal::ProposalId,
    review::{AdvisorReview, ReviewType},
    EventId,
};
use std::sync::Arc;

pub fn review(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/:event/:objective/:proposal/reviews",
            get({
                let state = state.clone();
                move |path, query| async {
                    handle_result(reviews_exec(path, query, state).await).await
                }
            }),
        )
        .route(
            "/:event/:objective/review_types",
            get(move |path, query| async {
                handle_result(review_types_exec(path, query, state).await).await
            }),
        )
}

async fn reviews_exec(
    Path((event, objective, proposal)): Path<(EventId, ObjectiveId, ProposalId)>,
    reviews_query: Query<LimitOffset>,
    state: Arc<State>,
) -> Result<Vec<AdvisorReview>, Error> {
    tracing::debug!(
        "reviews_query, event:{0} objective: {1}, proposal: {2}",
        event.0,
        objective.0,
        proposal.0,
    );

    let reviews = state
        .event_db
        .get_reviews(
            event,
            objective,
            proposal,
            reviews_query.limit,
            reviews_query.offset,
        )
        .await?;
    Ok(reviews)
}

async fn review_types_exec(
    Path((event, objective)): Path<(EventId, ObjectiveId)>,
    reviews_query: Query<ReviewsQuery>,
    state: Arc<State>,
) -> Result<Vec<ReviewType>, Error> {
    tracing::debug!(
        "review_types_query, event:{0} objective: {1}",
        event.0,
        objective.0,
    );

    let reviews = state
        .event_db
        .get_review_types(event, objective, reviews_query.limit, reviews_query.offset)
        .await?;
    Ok(reviews)
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
        http::{Request, StatusCode},
    };
    use event_db::types::event::review::Rating;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn reviews_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/{1}/{2}/reviews", 1, 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                AdvisorReview {
                    assessor: "assessor 1".to_string(),
                    ratings: vec![
                        Rating {
                            review_type: 1,
                            score: 10,
                            note: Some("note 1".to_string()),
                        },
                        Rating {
                            review_type: 2,
                            score: 15,
                            note: Some("note 2".to_string()),
                        },
                        Rating {
                            review_type: 5,
                            score: 20,
                            note: Some("note 3".to_string()),
                        }
                    ],
                },
                AdvisorReview {
                    assessor: "assessor 2".to_string(),
                    ratings: vec![],
                },
                AdvisorReview {
                    assessor: "assessor 3".to_string(),
                    ratings: vec![],
                },
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/{2}/reviews?limit={3}",
                1, 1, 1, 2
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                AdvisorReview {
                    assessor: "assessor 1".to_string(),
                    ratings: vec![
                        Rating {
                            review_type: 1,
                            score: 10,
                            note: Some("note 1".to_string()),
                        },
                        Rating {
                            review_type: 2,
                            score: 15,
                            note: Some("note 2".to_string()),
                        },
                        Rating {
                            review_type: 5,
                            score: 20,
                            note: Some("note 3".to_string()),
                        }
                    ],
                },
                AdvisorReview {
                    assessor: "assessor 2".to_string(),
                    ratings: vec![],
                },
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/{2}/reviews?offset={3}",
                1, 1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                AdvisorReview {
                    assessor: "assessor 2".to_string(),
                    ratings: vec![],
                },
                AdvisorReview {
                    assessor: "assessor 3".to_string(),
                    ratings: vec![],
                },
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/{2}/reviews?limit={3}&offset={4}",
                1, 1, 1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![AdvisorReview {
                assessor: "assessor 2".to_string(),
                ratings: vec![],
            },])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );
    }

    #[tokio::test]
    async fn review_types_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/{1}/review_types", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                ReviewType {
                    id: 1,
                    name: "impact".to_string(),
                    description: Some("Impact Rating".to_string()),
                    min: 0,
                    max: 5,
                    map: vec![],
                    note: None,
                    group: Some("review_group 1".to_string()),
                },
                ReviewType {
                    id: 2,
                    name: "feasibility".to_string(),
                    description: Some("Feasibility Rating".to_string()),
                    min: 0,
                    max: 5,
                    map: vec![],
                    note: Some(true),
                    group: Some("review_group 2".to_string()),
                },
                ReviewType {
                    id: 5,
                    name: "vpa_ranking".to_string(),
                    description: Some("VPA Ranking of the review".to_string()),
                    min: 0,
                    max: 3,
                    map: vec![
                        json!(
                            {"name":"Excellent","desc":"Excellent Review"}
                        )
                        .to_string(),
                        json!({"name":"Good","desc":"Could be improved."}).to_string(),
                        json!({"name":"FilteredOut","desc":"Exclude this review"}).to_string(),
                        json!({"name":"NA", "desc":"Not Applicable"}).to_string()
                    ],
                    note: Some(false),
                    group: None,
                }
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/review_types?limit={2}",
                1, 1, 2
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                ReviewType {
                    id: 1,
                    name: "impact".to_string(),
                    description: Some("Impact Rating".to_string()),
                    min: 0,
                    max: 5,
                    map: vec![],
                    note: None,
                    group: Some("review_group 1".to_string()),
                },
                ReviewType {
                    id: 2,
                    name: "feasibility".to_string(),
                    description: Some("Feasibility Rating".to_string()),
                    min: 0,
                    max: 5,
                    map: vec![],
                    note: Some(true),
                    group: Some("review_group 2".to_string()),
                },
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/review_types?offset={2}",
                1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                ReviewType {
                    id: 2,
                    name: "feasibility".to_string(),
                    description: Some("Feasibility Rating".to_string()),
                    min: 0,
                    max: 5,
                    map: vec![],
                    note: Some(true),
                    group: Some("review_group 2".to_string()),
                },
                ReviewType {
                    id: 5,
                    name: "vpa_ranking".to_string(),
                    description: Some("VPA Ranking of the review".to_string()),
                    min: 0,
                    max: 3,
                    map: vec![
                        json!(
                            {"name":"Excellent","desc":"Excellent Review"}
                        )
                        .to_string(),
                        json!({"name":"Good","desc":"Could be improved."}).to_string(),
                        json!({"name":"FilteredOut","desc":"Exclude this review"}).to_string(),
                        json!({"name":"NA", "desc":"Not Applicable"}).to_string()
                    ],
                    note: Some(false),
                    group: None,
                }
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/{1}/review_types?limit={2}&offset={3}",
                1, 1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![ReviewType {
                id: 2,
                name: "feasibility".to_string(),
                description: Some("Feasibility Rating".to_string()),
                min: 0,
                max: 5,
                map: vec![],
                note: Some(true),
                group: Some("review_group 2".to_string()),
            },])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );
    }
}
