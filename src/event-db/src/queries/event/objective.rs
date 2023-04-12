use crate::{
    error::Error,
    types::event::{
        objective::{Objective, ObjectiveDetails, ObjectiveSummary, ObjectiveType},
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
    const OBJECTIVES_QUERY: &'static str = "SELECT objective.id, objective.title, objective.description, objective_category.name, objective_category.description as objective_category_description
        FROM objective
        INNER JOIN objective_category on objective.category = objective_category.name
        WHERE objective.event = $1;";
}

#[async_trait]
impl ObjectiveQueries for EventDB {
    async fn get_objectives(
        &self,
        event: EventId,
        _limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<Objective>, Error> {
        let conn = self.pool.get().await?;

        let rows = conn.query(Self::OBJECTIVES_QUERY, &[&event.0]).await?;

        let mut objectives = Vec::new();
        for row in rows {
            let summary = ObjectiveSummary {
                id: row.try_get("id")?,
                objective_type: ObjectiveType {
                    id: row.try_get("name")?,
                    description: row.try_get("objective_category_description")?,
                },
                title: row.try_get("title")?,
            };
            let details = ObjectiveDetails {
                description: row.try_get("description")?,
                reward: None,
                choices: None,
                ballot: None,
                url: None,
                supplemental: None,
            };
            objectives.push(Objective { summary, details });
        }

        Ok(objectives)
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
                        id: 1,
                        objective_type: ObjectiveType {
                            id: "catalyst-simple".to_string(),
                            description: "A Simple choice".to_string()
                        },
                        title: "title 1".to_string(),
                    },
                    details: ObjectiveDetails {
                        description: "description 1".to_string(),
                        reward: None,
                        choices: None,
                        ballot: None,
                        url: None,
                        supplemental: None,
                    }
                },
                Objective {
                    summary: ObjectiveSummary {
                        id: 2,
                        objective_type: ObjectiveType {
                            id: "catalyst-native".to_string(),
                            description: "??".to_string()
                        },
                        title: "title 2".to_string(),
                    },
                    details: ObjectiveDetails {
                        description: "description 2".to_string(),
                        reward: None,
                        choices: None,
                        ballot: None,
                        url: None,
                        supplemental: None,
                    }
                }
            ]
        );
    }
}
