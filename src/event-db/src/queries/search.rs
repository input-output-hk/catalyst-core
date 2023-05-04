use crate::{types::search::SearchQuery, Error, EventDB};
use async_trait::async_trait;

#[async_trait]
pub trait SearchQueries: Sync + Send + 'static {
    async fn search(&self, search_query: SearchQuery) -> Result<(), Error>;
}

impl EventDB {
    const SEARCH_QUERY: &'static str = "SELECT * FROM $1;";
}

#[async_trait]
impl SearchQueries for EventDB {
    async fn search(&self, search_query: SearchQuery) -> Result<(), Error> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(Self::SEARCH_QUERY, &[&search_query.table.to_string()])
            .await?;

        Ok(())
    }
}
