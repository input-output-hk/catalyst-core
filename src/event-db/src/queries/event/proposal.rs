use crate::{
    error::Error,
    types::event::{objective::ObjectiveId, proposal::ProposalSummary},
    EventDB,
};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum VoterGroup {
    Direct,
    Rep,
}

#[async_trait]
pub trait ProposalQueries: Sync + Send + 'static {
    async fn get_proposals(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        voter_group: Option<VoterGroup>,
        obj_id: ObjectiveId,
    ) -> Result<Vec<ProposalSummary>, Error>;
}

impl EventDB {
    const PROPOSALS_QUERY: &'static str =
        "SELECT id, title, summary FROM proposal WHERE objective = $1 LIMIT $2 OFFSET $3;";
}

#[async_trait]
impl ProposalQueries for EventDB {
    async fn get_proposals(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        _voter_group: Option<VoterGroup>,
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
/// To do it you can use `cargo make local-event-db-setup`
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {

    use std::env;

    use super::*;
    use crate::establish_connection;

    #[tokio::test]
    async fn get_proposals_test() {
        env::set_var(
            "EVENT_DB_URL",
            "postgres://catalyst-event-dev:CHANGE_MEy@localhost/CatalystEventDev",
        );

        let event_db = establish_connection(None).await.unwrap();

        let proposal_summary = event_db
            .get_proposals(None, None, Some(VoterGroup::Direct), ObjectiveId(1))
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
