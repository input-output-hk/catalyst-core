use crate::{
    error::Error,
    types::event::{objective::ObjectiveId, review::AdvisorReview},
    types::event::{proposal::ProposalId, EventId},
    EventDB,
};
use async_trait::async_trait;

#[async_trait]
pub trait ReviewQueries: Sync + Send + 'static {
    async fn get_reviews(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AdvisorReview>, Error>;
}

impl EventDB {
    const REVIEWS_QUERY: &'static str = "SELECT proposal_review.assessor
        FROM proposal_review
        INNER JOIN proposal on proposal.row_id = proposal_review.proposal_id
        INNER JOIN objective on proposal.objective = objective.row_id
        WHERE objective.event = $1 AND proposal.objective = $2 AND proposal.id = $3
        LIMIT $3 OFFSET $4;";
}

#[async_trait]
impl ReviewQueries for EventDB {
    async fn get_reviews(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AdvisorReview>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                Self::REVIEWS_QUERY,
                &[
                    &event.0,
                    &objective.0,
                    &proposal.0,
                    &limit,
                    &offset.unwrap_or(0),
                ],
            )
            .await?;

        let mut reviews = vec![];
        for row in rows {
            reviews.push(AdvisorReview {
                assessor: row.try_get("assessor")?,
                ratings: vec![],
            });
        }

        Ok(reviews)
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

        let reviews = event_db
            .get_reviews(EventId(1), ObjectiveId(1), ProposalId(1), None, None)
            .await
            .unwrap();

        assert_eq!(
            vec![
                AdvisorReview {
                    assessor: "assessor 1".to_string(),
                    ratings: vec![],
                },
                AdvisorReview {
                    assessor: "assessor 2".to_string(),
                    ratings: vec![],
                },
                AdvisorReview {
                    assessor: "assessor 3".to_string(),
                    ratings: vec![],
                },
            ],
            reviews
        );
    }
}
