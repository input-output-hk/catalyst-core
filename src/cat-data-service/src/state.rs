use crate::cli::Error;
use event_db::{schema_check::SchemaVersion, EventDB};

pub struct State {
    pub event_db: EventDB,
}

impl State {
    pub async fn new(database_url: String) -> Result<Self, Error> {
        let event_db = event_db::establish_connection(Some(database_url.as_str())).await?;
        event_db.schema_version_check().await?;
        Ok(Self { event_db })
    }
}
