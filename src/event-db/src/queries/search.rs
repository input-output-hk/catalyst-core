use crate::{
    types::{
        event::{EventId, EventSummary},
        search::{SearchQuery, SearchResult, SearchTable, ValueResults},
    },
    Error, EventDB,
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};

#[async_trait]
pub trait SearchQueries: Sync + Send + 'static {
    async fn search(&self, search_query: SearchQuery) -> Result<SearchResult, Error>;
}

impl EventDB {
    const SEARCH_EVENTS_QUERY: &'static str =
        "SELECT event.row_id, event.name, event.start_time, event.end_time, snapshot.last_updated
        FROM event
        LEFT JOIN snapshot ON event.row_id = snapshot.event";
}

#[async_trait]
impl SearchQueries for EventDB {
    async fn search(&self, search_query: SearchQuery) -> Result<SearchResult, Error> {
        let conn = self.pool.get().await?;

        match search_query.table {
            SearchTable::Events => {
                let mut where_clause = String::new();
                let mut filter_iter = search_query.filter.iter();
                if let Some(filter) = filter_iter.next() {
                    where_clause.push_str(
                        format!(
                            "WHERE event.{0} LIKE '%{1}%'",
                            filter.column.to_string(),
                            filter.search
                        )
                        .as_str(),
                    );
                    for filter in filter_iter {
                        where_clause.push_str(
                            format!(
                                "AND event.{0} LIKE '%{1}%'",
                                filter.column.to_string(),
                                filter.search
                            )
                            .as_str(),
                        );
                    }
                }

                let mut order_by_clause = String::new();
                let mut order_by_iter = search_query.order_by.iter();
                if let Some(order_by) = order_by_iter.next() {
                    let order_type = if order_by.descending { "DESC" } else { "ASC" };
                    order_by_clause.push_str(
                        format!(
                            "ORDER BY event.{0} {1}",
                            order_by.column.to_string(),
                            order_type
                        )
                        .as_str(),
                    );
                    for order_by in order_by_iter {
                        let order_type = if order_by.descending { "DESC" } else { "ASC" };
                        order_by_clause.push_str(
                            format!(
                                ", event.{0} LIKE '%{1}%'",
                                order_by.column.to_string(),
                                order_type
                            )
                            .as_str(),
                        );
                    }
                }

                let rows: Vec<tokio_postgres::Row> = conn
                    .query(
                        &format!(
                            "{0} {1} {2};",
                            Self::SEARCH_EVENTS_QUERY,
                            where_clause,
                            order_by_clause
                        ),
                        &[],
                    )
                    .await
                    .map_err(|e| Error::NotFound(e.to_string()))?;

                let mut events = Vec::new();
                for row in rows {
                    let ends = row
                        .try_get::<&'static str, Option<NaiveDateTime>>("end_time")?
                        .map(|val| val.and_local_timezone(Utc).unwrap());
                    let is_final = ends.map(|ends| Utc::now() > ends).unwrap_or(false);
                    events.push(EventSummary {
                        id: EventId(row.try_get("row_id")?),
                        name: row.try_get("name")?,
                        starts: row
                            .try_get::<&'static str, Option<NaiveDateTime>>("start_time")?
                            .map(|val| val.and_local_timezone(Utc).unwrap()),
                        reg_checked: row
                            .try_get::<&'static str, Option<NaiveDateTime>>("last_updated")?
                            .map(|val| val.and_local_timezone(Utc).unwrap()),
                        ends,
                        is_final,
                    })
                }

                Ok(SearchResult {
                    total: events.len() as u32,
                    results: ValueResults::Event(events),
                })
            }
            SearchTable::Objectives => Ok(SearchResult {
                total: 0,
                results: ValueResults::Objective(vec![]),
            }),
            SearchTable::Proposals => Ok(SearchResult {
                total: 0,
                results: ValueResults::Proposal(vec![]),
            }),
        }
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
    use crate::{
        establish_connection,
        types::search::{SearchColumn, SearchConstraint, SearchOrderBy},
    };

    #[tokio::test]
    async fn search_events_test() {
        let event_db = establish_connection(None).await.unwrap();

        let search_query = SearchQuery {
            table: SearchTable::Events,
            filter: vec![SearchConstraint {
                column: SearchColumn::Desc,
                search: "Fund 4".to_string(),
            }],
            order_by: vec![SearchOrderBy {
                column: SearchColumn::Desc,
                descending: true,
            }],
        };
        let query_result = event_db.search(search_query).await.unwrap();
        assert_eq!(query_result.total, 1);
        assert_eq!(
            query_result.results,
            ValueResults::Event(vec![EventSummary {
                id: EventId(4),
                name: "Test Fund 4".to_string(),
                starts: None,
                ends: None,
                reg_checked: None,
                is_final: false,
            }])
        );

        let search_query = SearchQuery {
            table: SearchTable::Events,
            filter: vec![SearchConstraint {
                column: SearchColumn::Funds,
                search: "Fund 4".to_string(),
            }],
            order_by: vec![],
        };
        assert_eq!(
            event_db.search(search_query).await,
            Err(Error::NotFound(
                "db error: ERROR: column event.funds does not exist".to_string()
            ))
        )
    }
}
