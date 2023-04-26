use crate::{
    error::Error,
    types::event::{
        objective::ObjectiveId,
        proposal::{Proposal, ProposalId, ProposerDetails},
        EventId,
    },
    EventDB,
};
use async_trait::async_trait;

#[async_trait]
pub trait ProposalQueries: Sync + Send + 'static {
    async fn get_proposal(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
    ) -> Result<Proposal, Error>;
}

impl EventDB {
    const PROPOSAL_QUERY: &'static str = "SELECT proposal.funds, proposal.url, proposal.files_url,
    proposal.proposer_name, proposal.proposer_contact, proposal.proposer_url, proposal.public_key
    FROM proposal
    INNER JOIN objective on proposal.objective = objective.row_id
    WHERE proposal.id = $1 AND proposal.objective = $2 AND objective.event = $3;";
}

#[async_trait]
impl ProposalQueries for EventDB {
    async fn get_proposal(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
    ) -> Result<Proposal, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(Self::PROPOSAL_QUERY, &[&event.0, &objective.0, &proposal.0])
            .await?;
        let row = rows.get(0).ok_or(Error::NotFound)?;

        let _proposer = ProposerDetails {
            name: row.try_get("proposer_name")?,
            email: row.try_get("proposer_contact")?,
            url: row.try_get("proposer_url")?,
            payment_key: row.try_get("public_key")?,
        };

        Err(Error::NotFound)
    }
}

/// Need to setup and run a test event db instance
/// To do it you can use `cargo make local-event-db-test`
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {}
