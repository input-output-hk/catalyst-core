use crate::{
    error::Error,
    types::event::{
        objective::{
            Objective, ObjectiveDetails, ObjectiveId, ObjectiveSummary, ObjectiveSupplementalData,
            ObjectiveType, RewardDefintion,
        },
        EventId,
    },
    EventDB,
};
use async_trait::async_trait;

#[async_trait]
pub trait ObjectiveQueries: Sync + Send + 'static {
    async fn get_objectives(
        &self,
        event: EventId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Objective>, Error>;
}

impl EventDB {
    const OBJECTIVES_QUERY: &'static str =
        "SELECT objective.id, objective.title, objective.description, objective.rewards_currency, objective.rewards_total, objective.extra,
        objective_category.name, objective_category.description as objective_category_description
        FROM objective
        INNER JOIN objective_category on objective.category = objective_category.name
        WHERE objective.event = $1
        LIMIT $2 OFFSET $3;";
}

#[async_trait]
impl ObjectiveQueries for EventDB {
    async fn get_objectives(
        &self,
        event: EventId,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Objective>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                Self::OBJECTIVES_QUERY,
                &[&event.0, &limit, &offset.unwrap_or(0)],
            )
            .await?;

        let mut objectives = Vec::new();
        for row in rows {
            let summary = ObjectiveSummary {
                id: ObjectiveId(row.try_get("id")?),
                objective_type: ObjectiveType {
                    id: row.try_get("name")?,
                    description: row.try_get("objective_category_description")?,
                },
                title: row.try_get("title")?,
                description: row.try_get("description")?,
            };
            let currency: Option<_> = row.try_get("rewards_currency")?;
            let value: Option<_> = row.try_get("rewards_total")?;
            let reward = match (currency, value) {
                (Some(currency), Some(value)) => Some(RewardDefintion { currency, value }),
                _ => None,
            };
            let details = ObjectiveDetails {
                reward,
                supplemental: row
                    .try_get::<_, Option<serde_json::Value>>("extra")?
                    .map(ObjectiveSupplementalData),
            };
            objectives.push(Objective { summary, details });
        }

        Ok(objectives)
    }
}

/// Need to setup and run a test event db instance
/// To do it you can use the following commands:
/// Prepare docker images
/// ```
/// earthly ./containers/event-db-migrations+docker-test
/// ```
/// Run event-db container
/// ```
/// docker-compose -f src/event-db/docker-compose.yml up migrations-test
/// ```
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::establish_connection;

    #[tokio::test]
    async fn get_objectives_test() {
        let event_db = establish_connection(None).await.unwrap();

        let objectives = event_db
            .get_objectives(EventId(1), None, None)
            .await
            .unwrap();
        assert_eq!(
            objectives,
            vec![
                Objective {
                    summary: ObjectiveSummary {
                        id: ObjectiveId(1),
                        objective_type: ObjectiveType {
                            id: "catalyst-simple".to_string(),
                            description: "A Simple choice".to_string()
                        },
                        title: "title 1".to_string(),
                        description: "description 1".to_string(),
                    },
                    details: ObjectiveDetails {
                        reward: Some(RewardDefintion {
                            currency: "ADA".to_string(),
                            value: 100
                        }),
                        supplemental: Some(ObjectiveSupplementalData(json!(
                        {
                            "url": "objective 1 url",
                            "sponsor": "objective 1 sponsor",
                            "video": "objective 1 video"
                        }
                        ))),
                    }
                },
                Objective {
                    summary: ObjectiveSummary {
                        id: ObjectiveId(2),
                        objective_type: ObjectiveType {
                            id: "catalyst-native".to_string(),
                            description: "??".to_string()
                        },
                        title: "title 2".to_string(),
                        description: "description 2".to_string(),
                    },
                    details: ObjectiveDetails {
                        reward: None,
                        supplemental: None,
                    }
                }
            ]
        );

        let objectives = event_db
            .get_objectives(EventId(1), Some(1), None)
            .await
            .unwrap();
        assert_eq!(
            objectives,
            vec![Objective {
                summary: ObjectiveSummary {
                    id: ObjectiveId(1),
                    objective_type: ObjectiveType {
                        id: "catalyst-simple".to_string(),
                        description: "A Simple choice".to_string()
                    },
                    title: "title 1".to_string(),
                    description: "description 1".to_string(),
                },
                details: ObjectiveDetails {
                    reward: Some(RewardDefintion {
                        currency: "ADA".to_string(),
                        value: 100
                    }),
                    supplemental: Some(ObjectiveSupplementalData(json!(
                    {
                        "url": "objective 1 url",
                        "sponsor": "objective 1 sponsor",
                        "video": "objective 1 video"
                    }
                    ))),
                }
            },]
        );

        let objectives = event_db
            .get_objectives(EventId(1), None, Some(1))
            .await
            .unwrap();
        assert_eq!(
            objectives,
            vec![Objective {
                summary: ObjectiveSummary {
                    id: ObjectiveId(2),
                    objective_type: ObjectiveType {
                        id: "catalyst-native".to_string(),
                        description: "??".to_string()
                    },
                    title: "title 2".to_string(),
                    description: "description 2".to_string(),
                },
                details: ObjectiveDetails {
                    reward: None,
                    supplemental: None,
                }
            }]
        );

        let objectives = event_db
            .get_objectives(EventId(1), Some(1), Some(2))
            .await
            .unwrap();
        assert_eq!(objectives, vec![]);
    }
}
