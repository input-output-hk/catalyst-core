use crate::{
    error::Error,
    types::event::proposal::{EventId, ObjectId, ProposalSummary},
    EventDB,
};
use async_trait::async_trait;

pub enum VoterGroup {
    Direct,
    Rep,
}

#[async_trait]
pub trait ProposalQueries: Sync + Send + 'static {
    async fn get_proposals(
        &self,
        event: EventId,
        obj: ObjectId,
        limit: Option<i64>,
        offset: Option<i64>,
        voter_group: VoterGroup,
    ) -> Result<Vec<ProposalSummary>, Error>;
}

impl EventDB {
    const PROPOSALS_QUERY: &'static str =
    "SELECT objective.id, objective.title, objective.description, objective.rewards_currency, objective.rewards_total, objective.extra,
    objective_category.name, objective_category.description as objective_category_description,
    vote_options.objective as choices
    FROM objective";
}

#[async_trait]
impl ProposalQueries for EventDB {
    async fn get_proposals(
        &self,
        event: EventId,
        obj: ObjectId,
        _limit: Option<i64>,
        _offset: Option<i64>,
        _voter_group: VoterGroup,
    ) -> Result<Vec<ProposalSummary>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn.query(Self::PROPOSALS_QUERY, &[&event.0]).await?;

        let mut proposals = Vec::new();
        for row in rows {
            let summary = ProposalSummary {
                id: EventId(row.try_get("id")?),
                name: row.try_get("name")?,
                summary: String::from("summary"),
                starts: None,
                ends: None,
                reg_checked: None,
                is_final: false,
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
            "postgres://catalyst-event-dev:magnify@localhost/CatalystEventDev",
        );

        let event_db = establish_connection(None).await.unwrap();

        let proposals = event_db
            .get_proposals(EventId(1), ObjectId(1), None, None, VoterGroup::Direct)
            .await
            .unwrap();

        println!("{:?}", proposals);
    }
}
