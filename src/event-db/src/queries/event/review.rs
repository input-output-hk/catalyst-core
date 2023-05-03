use crate::{
    error::Error,
    types::event::{
        objective::ObjectiveId,
        review::{AdvisorReview, Rating, ReviewType},
    },
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

    async fn get_review_types(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ReviewType>, Error>;
}

impl EventDB {
    const REVIEWS_QUERY: &'static str = "SELECT proposal_review.row_id, proposal_review.assessor
        FROM proposal_review
        INNER JOIN proposal on proposal.row_id = proposal_review.proposal_id
        INNER JOIN objective on proposal.objective = objective.row_id
        WHERE objective.event = $1 AND proposal.objective = $2 AND proposal.id = $3
        LIMIT $4 OFFSET $5;";

    const RATINGS_PER_REVIEW_QUERY: &'static str =
        "SELECT review_rating.metric, review_rating.rating, review_rating.note
        FROM review_rating
        WHERE review_rating.review_id = $1;";

    const REVIEW_TYPES_QUERY: &'static str =
        "SELECT review_metric.row_id, review_metric.name, review_metric.description,
        review_metric.min, review_metric.max, review_metric.map
        FROM review_metric
        INNER JOIN review_rating on review_metric.row_id = review_rating.metric
        INNER JOIN proposal_review on review_rating.review_id = proposal_review.row_id
        INNER JOIN proposal on proposal.row_id = proposal_review.proposal_id
        INNER JOIN objective on proposal.objective = objective.row_id
        WHERE objective.event = $1 AND proposal.objective = $2 AND proposal.id = $3
        LIMIT $4 OFFSET $5;";
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

        let mut reviews = Vec::new();
        for row in rows {
            let assessor = row.try_get("assessor")?;
            let review_id: i32 = row.try_get("row_id")?;

            let mut ratings = Vec::new();
            let rows = conn
                .query(Self::RATINGS_PER_REVIEW_QUERY, &[&review_id])
                .await?;
            for row in rows {
                ratings.push(Rating {
                    review_type: row.try_get("metric")?,
                    score: row.try_get("rating")?,
                    note: row.try_get("note")?,
                })
            }

            reviews.push(AdvisorReview { assessor, ratings })
        }

        Ok(reviews)
    }

    async fn get_review_types(
        &self,
        event: EventId,
        objective: ObjectiveId,
        proposal: ProposalId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ReviewType>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                Self::REVIEW_TYPES_QUERY,
                &[
                    &event.0,
                    &objective.0,
                    &proposal.0,
                    &limit,
                    &offset.unwrap_or(0),
                ],
            )
            .await?;
        let mut review_types = Vec::new();
        for row in rows {
            review_types.push(ReviewType {
                id: row.try_get("row_id")?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                min: row.try_get("min")?,
                max: row.try_get("max")?,
                note: None,
                map: vec![],
                group: "group".to_string(),
            })
        }

        Ok(review_types)
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
    async fn get_reviews_test() {
        let event_db = establish_connection(None).await.unwrap();

        let reviews = event_db
            .get_reviews(EventId(1), ObjectiveId(1), ProposalId(1), None, None)
            .await
            .unwrap();

        assert_eq!(
            vec![
                AdvisorReview {
                    assessor: "assessor 1".to_string(),
                    ratings: vec![
                        Rating {
                            review_type: 1,
                            score: 10,
                            note: Some("note 1".to_string()),
                        },
                        Rating {
                            review_type: 2,
                            score: 15,
                            note: Some("note 2".to_string()),
                        },
                        Rating {
                            review_type: 3,
                            score: 20,
                            note: Some("note 3".to_string()),
                        }
                    ],
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

        let reviews = event_db
            .get_reviews(EventId(1), ObjectiveId(1), ProposalId(1), Some(2), None)
            .await
            .unwrap();

        assert_eq!(
            vec![
                AdvisorReview {
                    assessor: "assessor 1".to_string(),
                    ratings: vec![
                        Rating {
                            review_type: 1,
                            score: 10,
                            note: Some("note 1".to_string()),
                        },
                        Rating {
                            review_type: 2,
                            score: 15,
                            note: Some("note 2".to_string()),
                        },
                        Rating {
                            review_type: 3,
                            score: 20,
                            note: Some("note 3".to_string()),
                        }
                    ],
                },
                AdvisorReview {
                    assessor: "assessor 2".to_string(),
                    ratings: vec![],
                },
            ],
            reviews
        );

        let reviews = event_db
            .get_reviews(EventId(1), ObjectiveId(1), ProposalId(1), None, Some(1))
            .await
            .unwrap();

        assert_eq!(
            vec![
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

        let reviews = event_db
            .get_reviews(EventId(1), ObjectiveId(1), ProposalId(1), Some(1), Some(1))
            .await
            .unwrap();

        assert_eq!(
            vec![AdvisorReview {
                assessor: "assessor 2".to_string(),
                ratings: vec![],
            },],
            reviews
        );
    }

    #[tokio::test]
    async fn get_review_types_test() {
        let event_db = establish_connection(None).await.unwrap();

        let review_types = event_db
            .get_review_types(EventId(1), ObjectiveId(1), ProposalId(1), None, None)
            .await
            .unwrap();

        assert_eq!(
            vec![
                ReviewType {
                    id: 1,
                    name: "impact".to_string(),
                    description: "Impact Rating".to_string(),
                    min: 0,
                    max: 5,
                    note: None,
                    map: vec![],
                    group: "group".to_string(),
                },
                ReviewType {
                    id: 2,
                    name: "feasibility".to_string(),
                    description: "Feasibility Rating".to_string(),
                    min: 0,
                    max: 5,
                    note: None,
                    map: vec![],
                    group: "group".to_string(),
                },
                ReviewType {
                    id: 3,
                    name: "auditability".to_string(),
                    description: "Auditability Rating".to_string(),
                    min: 0,
                    max: 5,
                    note: None,
                    map: vec![],
                    group: "group".to_string(),
                }
            ],
            review_types
        );
    }
}
