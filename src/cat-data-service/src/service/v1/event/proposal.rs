use crate::{
    service::{handle_result, Error},
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
use serde::Deserialize;
use std::sync::Arc;

pub fn proposal(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/:event/:objective/proposals",
            get({
                let state = state.clone();
                move |path, query| async {
                    handle_result(proposals_exec(path, query, state).await).await
                }
            }),
        )
        .route(
            "/:event/:objective/:proposal/proposal",
            get({
                let state = state.clone();
                move |path| async { handle_result(proposal_exec(path, state).await).await }
            }),
        )
}

#[derive(Deserialize)]
struct ProposalsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn proposals_exec(
    Path((event, objective)): Path<(EventId, ObjectiveId)>,
    proposals_query: Query<ProposalsQuery>,
    state: Arc<State>,
) -> Result<Vec<ProposalSummary>, Error> {
    tracing::debug!(
        "proposals_query, event:{0} objective: {1}",
        event.0,
        objective.0
    );

    let event = state
        .event_db
        .get_proposals(
            event,
            objective,
            proposals_query.limit,
            proposals_query.offset,
        )
        .await?;
    Ok(event)
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

    let event = state
        .event_db
        .get_proposal(event, objective, proposal)
        .await?;
    Ok(event)
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
    use event_db::types::event::proposal::{ProposalDetails, ProposerDetails};
    use tower::ServiceExt;

    #[tokio::test]
    async fn proposal_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/{1}/{2}/proposal", 1, 1, 1))
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
                    ballot: None,
                    supplemental: None,
                }
            })
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/{1}/{2}/proposal", 3, 3, 3))
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
            .uri(format!("/api/v1/event/{0}/{1}/proposals", 1, 1))
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
                "/api/v1/event/{0}/{1}/proposals?limit={2}",
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
                "/api/v1/event/{0}/{1}/proposals?offset={2}",
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
                "/api/v1/event/{0}/{1}/proposals?offset={2}&limit={3}",
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
