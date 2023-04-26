use crate::{
    error::Error,
    types::event::{objective::ObjectiveId, proposal::ProposalSummary},
    types::event::{
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

    async fn get_proposals(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        // TODO: Voter Group: future state may require dreps
        obj_id: ObjectiveId,
    ) -> Result<Vec<ProposalSummary>, Error>;
}

impl EventDB {
    const PROPOSALS_QUERY: &'static str =
        "SELECT id, title, summary FROM proposal WHERE objective = $1 LIMIT $2 OFFSET $3;";

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

    async fn get_proposals(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        obj_id: ObjectiveId,
    ) -> Result<Vec<ProposalSummary>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                Self::PROPOSALS_QUERY,
                &[&obj_id.0, &limit, &offset.unwrap_or(0)],
            )
            .await?;

        let mut proposals = Vec::new();
        for row in rows {
            let summary = ProposalSummary {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                summary: row.try_get("summary")?,
            };

            proposals.push(summary);
        }

        Ok(proposals)
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
mod tests {
    use super::*;
    use crate::establish_connection;

    #[tokio::test]
    async fn get_proposals_test() {
        let event_db = establish_connection(None).await.unwrap();

        let proposal_summary = event_db
            .get_proposals(None, None, ObjectiveId(1))
            .await
            .unwrap();

        assert_eq!(
            vec![
                ProposalSummary {
                    id: 1,
                    title: String::from("title 1"),
                    summary: String::from("summary 1")
                },
                ProposalSummary {
                    id: 3,
                    title: String::from("title 3"),
                    summary: String::from("summary 3")
                }
            ],
            proposal_summary
        )
    }
}
