use crate::cli::Error;
use event_db::queries::snapshot::SnapshotQueries;
use std::sync::Arc;

pub struct State {
    pub event_db: Arc<dyn SnapshotQueries>,
}

impl State {
    pub async fn new(database_url: String) -> Result<Self, Error> {
        let event_db = Arc::new(event_db::establish_connection(Some(database_url.as_str())).await?);
        Ok(Self { event_db })
    }
}
