//! Check if the schema is up-to-date.
use crate::{Error, EventDB, DATABASE_SCHEMA_VERSION};
use async_trait::async_trait;

/// Trait to check the schema version of a connection.
#[async_trait]
pub trait SchemaVersion {
    /// Check the schema version.
    /// return the current schema version if its current.
    /// Otherwise return an error.
    async fn schema_version_check(&self) -> Result<i32, Error>;
}

#[async_trait]
impl SchemaVersion for EventDB {
    async fn schema_version_check(&self) -> Result<i32, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let schema_check = conn
            .query_one("SELECT MAX(version) FROM refinery_schema_history;", &[])
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let current_ver = schema_check
            .try_get("max")
            .map_err(|err| Error::Unknown(err.to_string()))?;

        if current_ver == DATABASE_SCHEMA_VERSION {
            Ok(current_ver)
        } else {
            Err(Error::MismatchedSchema {
                was: current_ver,
                expected: DATABASE_SCHEMA_VERSION,
            })
        }
    }
}
