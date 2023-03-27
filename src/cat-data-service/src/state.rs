use crate::cli::Error;
use event_db::{queries::snapshot::SnapshotQueries, schema_check::SchemaVersion, EventDB};

pub struct State<EventDB: SnapshotQueries> {
    pub event_db: EventDB,
}

impl State<EventDB> {
    pub async fn new(database_url: String) -> Result<Self, Error> {
        let event_db = event_db::establish_connection(Some(database_url.as_str())).await?;
        event_db.schema_version_check().await?;
        Ok(Self { event_db })
    }
}
