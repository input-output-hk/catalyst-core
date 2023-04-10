use crate::{
    error::Error,
    types::event::{Event, EventId, EventSummary},
    EventDB,
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};

#[async_trait]
pub trait EventQueries: Sync + Send + 'static {
    async fn get_events(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<EventSummary>, Error>;
    async fn get_event(&self, event: EventId) -> Result<Event, Error>;
}

impl EventDB {
    const EVENTS_QUERY: &'static str =
        "SELECT event.row_id, event.name, event.start_time, event.end_time, snapshot.final
        FROM event
        INNER JOIN snapshot ON event.row_id = snapshot.event
        ORDER BY event.row_id ASC
        OFFSET $1;";

    const EVENTS_WITH_LIMIT_QUERY: &'static str =
        "SELECT event.row_id, event.name, event.start_time, event.end_time, snapshot.final
        FROM event
        INNER JOIN snapshot ON event.row_id = snapshot.event
        ORDER BY event.row_id ASC
        LIMIT $1 OFFSET $2";

    const EVENT_QUERY: &'static str =
        "SELECT event.row_id, event.name, event.start_time, event.end_time, snapshot.final
        FROM event
        INNER JOIN snapshot ON event.row_id = snapshot.event
        WHERE event.row_id = $1;";
}

#[async_trait]
impl EventQueries for EventDB {
    async fn get_events(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<EventSummary>, Error> {
        let conn = self.pool.get().await?;

        let rows = if let Some(limit) = limit {
            conn.query(
                Self::EVENTS_WITH_LIMIT_QUERY,
                &[&limit, &offset.unwrap_or(0)],
            )
            .await?
        } else {
            conn.query(Self::EVENTS_QUERY, &[&offset.unwrap_or(0)])
                .await?
        };

        let mut events = Vec::new();
        for row in rows {
            events.push(EventSummary {
                id: EventId(row.try_get("row_id")?),
                name: row.try_get("name")?,
                starts: row
                    .try_get::<&'static str, NaiveDateTime>("start_time")?
                    .and_local_timezone(Utc)
                    .unwrap(),
                ends: row
                    .try_get::<&'static str, NaiveDateTime>("end_time")?
                    .and_local_timezone(Utc)
                    .unwrap(),
                is_final: row.try_get("final")?,
                reg_checked: None,
            })
        }

        Ok(events)
    }

    async fn get_event(&self, event: EventId) -> Result<Event, Error> {
        let conn = self.pool.get().await?;

        let rows = conn.query(Self::EVENT_QUERY, &[&event.0]).await?;
        let row = rows.get(0).ok_or(Error::NotFound)?;

        Ok(Event {
            event_summary: EventSummary {
                id: EventId(row.try_get("row_id")?),
                name: row.try_get("name")?,
                starts: row
                    .try_get::<&'static str, NaiveDateTime>("start_time")?
                    .and_local_timezone(Utc)
                    .unwrap(),
                ends: row
                    .try_get::<&'static str, NaiveDateTime>("end_time")?
                    .and_local_timezone(Utc)
                    .unwrap(),
                is_final: row.try_get("final")?,
                reg_checked: None,
            },
            event_details: None,
        })
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
    use chrono::{DateTime, NaiveDate, NaiveTime};

    #[tokio::test]
    async fn get_events_test() {
        let event_db = establish_connection(None).await.unwrap();

        let events = event_db.get_events(None, None).await.unwrap();
        assert_eq!(
            events,
            vec![
                EventSummary {
                    id: EventId(1),
                    name: "Test Fund 1".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(2),
                    name: "Test Fund 2".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(3),
                    name: "Test Fund 3".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                }
            ]
        );

        let events = event_db.get_events(Some(2), None).await.unwrap();
        assert_eq!(
            events,
            vec![
                EventSummary {
                    id: EventId(1),
                    name: "Test Fund 1".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                EventSummary {
                    id: EventId(2),
                    name: "Test Fund 2".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
            ]
        );

        let events = event_db.get_events(Some(1), Some(1)).await.unwrap();
        assert_eq!(
            events,
            vec![EventSummary {
                id: EventId(2),
                name: "Test Fund 2".to_string(),
                starts: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2021, 5, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                ends: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2021, 6, 1).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
                reg_checked: None,
            },]
        );

        assert_eq!(
            event_db.get_events(Some(1), Some(10)).await.unwrap(),
            vec![]
        );
    }

    #[tokio::test]
    async fn get_event_test() {
        let event_db = establish_connection(None).await.unwrap();

        let event = event_db.get_event(EventId(1)).await.unwrap();
        assert_eq!(
            event,
            Event {
                event_summary: EventSummary {
                    id: EventId(1),
                    name: "Test Fund 1".to_string(),
                    starts: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 5, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    ends: DateTime::<Utc>::from_utc(
                        NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2020, 6, 1).unwrap(),
                            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                        ),
                        Utc
                    ),
                    is_final: true,
                    reg_checked: None,
                },
                event_details: None,
            },
        );

        assert_eq!(event_db.get_event(EventId(10)).await, Err(Error::NotFound));
    }
}
