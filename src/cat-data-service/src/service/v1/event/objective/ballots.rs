use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use event_db::types::event::{ballot::ProposalBallot, objective::ObjectiveId, EventId};
use std::sync::Arc;

pub fn ballots(state: Arc<State>) -> Router {
    Router::new().route(
        "/ballots",
        get(move |path| async { handle_result(ballots_exec(path, state).await).await }),
    )
}

async fn ballots_exec(
    Path((event, objective)): Path<(EventId, ObjectiveId)>,
    state: Arc<State>,
) -> Result<Vec<ProposalBallot>, Error> {
    tracing::debug!(
        "ballots_query, event: {0}, objective: {1}",
        event.0,
        objective.0,
    );

    let ballot = state
        .event_db
        .get_objective_ballots(event, objective)
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
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn ballots_test() {
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/objective/{1}/ballots", 1, 1))
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
                        "proposal_id": 10,
                        "ballot": {
                            "choices": ["yes", "no"],
                            "voteplans": [
                                {
                                    "chain_proposal_index": 10,
                                    "group": "direct",
                                    "ballot_type": "public",
                                    "chain_voteplan_id": "1",
                                },
                                {
                                    "chain_proposal_index": 12,
                                    "group": "rep",
                                    "ballot_type": "public",
                                    "chain_voteplan_id": "2",
                                }
                            ]
                        }
                    },
                    {
                        "proposal_id": 20,
                        "ballot": {
                            "choices": ["yes", "no"],
                            "voteplans": [
                                {
                                    "chain_proposal_index": 11,
                                    "group": "direct",
                                    "ballot_type": "public",
                                    "chain_voteplan_id": "1",
                                },
                                {
                                    "chain_proposal_index": 13,
                                    "group": "rep",
                                    "ballot_type": "public",
                                    "chain_voteplan_id": "2",
                                }
                            ]
                        }
                    },
                    {
                        "proposal_id": 30,
                        "ballot": {
                            "choices": ["yes", "no"],
                            "voteplans": []
                        }
                    }
                ]
            )
        );
    }
}
