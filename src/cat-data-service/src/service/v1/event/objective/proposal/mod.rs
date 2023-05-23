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
    proposal::{Proposal, ProposalId, ProposalSummary},
    EventId,
};
use std::sync::Arc;

mod review;

pub fn proposal(state: Arc<State>) -> Router {
    let review = review::review(state.clone());

    Router::new()
        .nest(
            "/proposal/:proposal",
            Router::new()
                .route(
                    "/",
                    get({
                        let state = state.clone();
                        move |path| async { handle_result(proposal_exec(path, state).await).await }
                    }),
                )
                .merge(review),
        )
        .route(
            "/proposals",
            get(move |path, query| async {
                handle_result(proposals_exec(path, query, state).await).await
            }),
        )
}

async fn proposals_exec(
    Path((event, objective)): Path<(EventId, ObjectiveId)>,
    lim_ofs: Query<LimitOffset>,
    state: Arc<State>,
) -> Result<Vec<ProposalSummary>, Error> {
    tracing::debug!(
        "proposals_query, event:{0} objective: {1}",
        event.0,
        objective.0
    );

    let proposals = state
        .event_db
        .get_proposals(event, objective, lim_ofs.limit, lim_ofs.offset)
        .await?;
    Ok(proposals)
}

async fn proposal_exec(
    Path((event, objective, proposal)): Path<(EventId, ObjectiveId, ProposalId)>,
    state: Arc<State>,
) -> Result<Proposal, Error> {
    tracing::debug!(
        "proposal_query, event:{0} objective: {1}, proposal: {2}",
        event.0,
        objective.0,
        proposal.0,
    );

    let proposal = state
        .event_db
        .get_proposal(event, objective, proposal)
        .await?;
    Ok(proposal)
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
    use event_db::types::event::proposal::{
        ProposalDetails, ProposalSupplementalDetails, ProposerDetails,
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn proposal_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposal/{2}",
                1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&Proposal {
                proposal_summary: ProposalSummary {
                    id: 1,
                    title: String::from("title 1"),
                    summary: String::from("summary 1")
                },
                proposal_details: ProposalDetails {
                    funds: 100,
                    url: "url.xyz".to_string(),
                    files: "files.xyz".to_string(),
                    proposer: vec![ProposerDetails {
                        name: "alice".to_string(),
                        email: "alice@io".to_string(),
                        url: "alice.prop.xyz".to_string(),
                        payment_key:
                            "b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde"
                                .to_string()
                    }],
                    supplemental: Some(ProposalSupplementalDetails(json!(
                        {
                            "brief": "Brief explanation of a proposal",
                            "goal": "The goal of the proposal is addressed to meet",
                            "importance": "The importance of the proposal",
                        }
                    ))),
                }
            })
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposal/{2}",
                3, 3, 3
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn proposals_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objective/{1}/proposals", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
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
                }
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposals?limit={2}",
                1, 1, 2
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
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
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposals?offset={2}",
                1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![
                ProposalSummary {
                    id: 2,
                    title: String::from("title 2"),
                    summary: String::from("summary 2")
                },
                ProposalSummary {
                    id: 3,
                    title: String::from("title 3"),
                    summary: String::from("summary 3")
                }
            ])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposals?offset={2}&limit={3}",
                1, 1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&vec![ProposalSummary {
                id: 2,
                title: String::from("title 2"),
                summary: String::from("summary 2")
            },])
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );
    }
}
