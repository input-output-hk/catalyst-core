use crate::{
    error::Error,
    types::event::{
        objective::ObjectiveId,
        proposal::{ProposalDetails, ProposalSummary, ProposalSupplementalDetails},
    },
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
        // TODO: Voter Group: future state may require dreps
        event: EventId,
        obj_id: ObjectiveId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ProposalSummary>, Error>;
}

impl EventDB {
    const PROPOSALS_QUERY: &'static str = "SELECT proposal.id, proposal.title, proposal.summary
        FROM proposal
        INNER JOIN objective on proposal.objective = objective.row_id
        WHERE proposal.objective = $1 AND objective.event = $2
        LIMIT $3 OFFSET $4;";

    const PROPOSAL_QUERY: &'static str =
        "SELECT proposal.id, proposal.title, proposal.summary, proposal.extra,
    proposal.funds, proposal.url, proposal.files_url,
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
        let row = rows
            .get(0)
            .ok_or(Error::NotFound("cat not find proposal value".to_string()))?;

        let proposer = vec![ProposerDetails {
            name: row.try_get("proposer_name")?,
            email: row.try_get("proposer_contact")?,
            url: row.try_get("proposer_url")?,
            payment_key: row.try_get("public_key")?,
        }];

        let extra = row.try_get::<_, Option<serde_json::Value>>("extra")?;
        let solution = extra
            .as_ref()
            .and_then(|extra| {
                extra
                    .get("solution")
                    .map(|solution| solution.as_str().map(|str| str.to_string()))
            })
            .flatten();
        let brief = extra
            .as_ref()
            .and_then(|extra| {
                extra
                    .get("brief")
                    .map(|brief| brief.as_str().map(|str| str.to_string()))
            })
            .flatten();
        let importance = extra
            .as_ref()
            .and_then(|val| {
                val.get("importance")
                    .map(|importance| importance.as_str().map(|str| str.to_string()))
            })
            .flatten();
        let metrics = extra
            .and_then(|val| {
                val.get("metrics")
                    .map(|metrics| metrics.as_str().map(|str| str.to_string()))
            })
            .flatten();
        let supplemental = match (solution, brief, importance, metrics) {
            (Some(solution), Some(brief), Some(importance), Some(metrics)) => {
                Some(ProposalSupplementalDetails {
                    solution,
                    brief,
                    importance,
                    metrics,
                })
            }
            _ => None,
        };

        let proposal_summary = ProposalSummary {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            summary: row.try_get("summary")?,
        };

        let proposal_details = ProposalDetails {
            proposer,
            supplemental,
            funds: row.try_get("funds")?,
            url: row.try_get("url")?,
            files: row.try_get("files_url")?,
            // TODO: fill the correct ballot data
            ballot: None,
        };

        Ok(Proposal {
            proposal_details,
            proposal_summary,
        })
    }

    async fn get_proposals(
        &self,
        event: EventId,
        objective: ObjectiveId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ProposalSummary>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                Self::PROPOSALS_QUERY,
                &[&objective.0, &event.0, &limit, &offset.unwrap_or(0)],
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
    async fn get_proposal_test() {
        let event_db = establish_connection(None).await.unwrap();

        let proposal = event_db
            .get_proposal(EventId(1), ObjectiveId(1), ProposalId(1))
            .await
            .unwrap();

        assert_eq!(
            Proposal {
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
            },
            proposal
        );
    }

    #[tokio::test]
    async fn get_proposals_test() {
        let event_db = establish_connection(None).await.unwrap();

        let proposal_summary = event_db
            .get_proposals(EventId(1), ObjectiveId(1), None, None)
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
                    id: 2,
                    title: String::from("title 2"),
                    summary: String::from("summary 2")
                },
                ProposalSummary {
                    id: 3,
                    title: String::from("title 3"),
                    summary: String::from("summary 3")
                }
            ],
            proposal_summary
        );

        let proposal_summary = event_db
            .get_proposals(EventId(1), ObjectiveId(1), Some(2), None)
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
                    id: 2,
                    title: String::from("title 2"),
                    summary: String::from("summary 2")
                },
            ],
            proposal_summary
        );

        let proposal_summary = event_db
            .get_proposals(EventId(1), ObjectiveId(1), None, Some(1))
            .await
            .unwrap();

        assert_eq!(
            vec![
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
            ],
            proposal_summary
        );

        let proposal_summary = event_db
            .get_proposals(EventId(1), ObjectiveId(1), Some(1), Some(1))
            .await
            .unwrap();

        assert_eq!(
            vec![ProposalSummary {
                id: 2,
                title: String::from("title 2"),
                summary: String::from("summary 2")
            },],
            proposal_summary
        );
    }
}
