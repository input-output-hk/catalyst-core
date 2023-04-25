use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use event_db::{
    queries::event::proposal::VoterGroup,
    types::event::{objective::ObjectiveId, proposal::ProposalSummary, EventId},
};
use serde::Deserialize;
use std::sync::Arc;

pub fn proposal(_state: Arc<State>) -> Router {
    Router::new()
}

pub fn proposals(state: Arc<State>) -> Router {
    Router::new().route(
        "/:event/:objective/proposals",
        get(move |path, query| async {
            handle_result(proposals_exec(path, query, state).await).await
        }),
    )
}

#[derive(Deserialize)]
struct ProposalsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    _voter_group: Option<VoterGroup>,
}

async fn proposals_exec(
    Path((event, objective)): Path<(EventId, ObjectiveId)>,
    proposals_query: Query<ProposalsQuery>,
    state: Arc<State>,
) -> Result<Vec<ProposalSummary>, Error> {
    tracing::debug!("proposals_query, event: {0}", event.0);

    let event = state
        .event_db
        .get_proposals(
            proposals_query.limit,
            proposals_query.offset,
            None,
            objective,
        )
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
    use std::env;

    use super::*;
    use crate::service::app;
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };

    use tower::ServiceExt;

    #[tokio::test]
    async fn proposals_test() {
        env::set_var(
            "EVENT_DB_URL",
            "postgres://catalyst-event-dev:CHANGE_MEy@localhost/CatalystEventDev",
        );
        let state = Arc::new(State::new(None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri(format!("/api/v1/event/{0}/{1}/proposals", 1, 1))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let proposals =
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap();

        assert_eq!(
            serde_json::to_string(&vec![
                ProposalSummary {
                    id: 1,
                    title: String::from("title 1"),
                    summary: String::from("summary 1")
                },
                ProposalSummary {
                    id: 4,
                    title: String::from("title 3"),
                    summary: String::from("summary 3")
                }
            ])
            .unwrap(),
            proposals
        )
    }
}
