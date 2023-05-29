use crate::{
    error::Error,
    types::event::{
        ballot::{Ballot, GroupVotePlans, ObjectiveChoices},
        objective::ObjectiveId,
        proposal::ProposalId,
        EventId,
    },
    EventDB,
};
use async_trait::async_trait;

#[async_trait]
pub trait BallotQueries: Sync + Send + 'static {
    async fn get_ballot(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
    ) -> Result<Ballot, Error>;
}

impl EventDB {
    const BALLOT_BY_EVENT_OBJECTIVE_PROPOSAL_QUERY: &'static str = "SELECT *
        FROM proposal
        LEFT INNER JOIN proposal ON proposal.objective = objective.row_id
        LEFT INNER JOIN objective.event ON objective.event = event.row_id
        WHERE event.row_id = $1 AND objective.row_id = $2 AND proposal.row_id = $2;";
}

#[async_trait]
impl BallotQueries for EventDB {
    async fn get_ballot(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
    ) -> Result<Ballot, Error> {
        Ok(Ballot {
            choices: ObjectiveChoices(vec![]),
            voteplans: GroupVotePlans(vec![]),
        })
    }
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
mod tests {}
