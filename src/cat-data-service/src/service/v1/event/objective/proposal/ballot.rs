use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use event_db::types::event::{
    ballot::Ballot, objective::ObjectiveId, proposal::ProposalId, EventId,
};
use std::sync::Arc;

pub fn ballot(state: Arc<State>) -> Router {
    Router::new().route(
        "/ballot",
        get(move |path| async { handle_result(ballot_exec(path, state).await).await }),
    )
}

async fn ballot_exec(
    Path((event, objective, proposal)): Path<(EventId, ObjectiveId, ProposalId)>,
    state: Arc<State>,
) -> Result<Ballot, Error> {
    tracing::debug!(
        "ballot_query, event: {0}, objective: {1}, proposal: {2}",
        event.0,
        objective.0,
        proposal.0,
    );

    let ballot = state
        .event_db
        .get_ballot(event, objective, proposal)
        .await?;
    Ok(ballot)
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
        event::ballot::{BallotType, GroupVotePlans, ObjectiveChoices, VotePlan},
        registration::VoterGroupId,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn ballot_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposal/{2}/ballot",
                1, 1, 1
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            serde_json::to_string(&Ballot {
                choices: ObjectiveChoices(vec!["yes".to_string(), "no".to_string()]),
                voteplans: GroupVotePlans(vec![
                    VotePlan {
                        chain_proposal_index: 10,
                        group: VoterGroupId("direct".to_string()),
                        ballot_type: BallotType("public".to_string()),
                        chain_voteplan_id: "1".to_string(),
                        encryption_key: None,
                    },
                    VotePlan {
                        chain_proposal_index: 12,
                        group: VoterGroupId("rep".to_string()),
                        ballot_type: BallotType("public".to_string()),
                        chain_voteplan_id: "2".to_string(),
                        encryption_key: None,
                    }
                ]),
            })
            .unwrap(),
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
        );

        let request = Request::builder()
            .uri(format!(
                "/api/v1/event/{0}/objective/{1}/proposal/{2}/ballot",
                3, 3, 3
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
