use crate::{
    error::Error,
    types::event::{
        objective::{Objective, ObjectiveSummary, ObjectiveType},
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
    const OBJECTIVES_QUERY: &'static str = "SELECT objective.id, objective.title, objective_category.name, objective_category.description 
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

        let mut objects = Vec::new();
        for row in rows {
            let objective_summary = ObjectiveSummary {
                id: row.try_get("id")?,
                objective_type: ObjectiveType {
                    id: row.try_get::<&'static str, String>("name")?.try_into()?,
                    description: row.try_get("description")?,
                },
                title: row.try_get("title")?,
            };
        }

        Ok(objects)
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
        assert_eq!(objectives, vec![]);
    }
}
