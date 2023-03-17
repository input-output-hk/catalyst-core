//! Check if the schema is up-to-date.

use std::{error::Error, fmt};

use async_trait::async_trait;

use crate::{EventDB, DATABASE_SCHEMA_VERSION};

/// Schema in database does not match schema supported by the Crate.
#[derive(Debug)]
struct MismatchedSchema {
    /// The current schema version.
    was: u32,
    /// The schema version we expected.
    expected: u32,
}

impl Error for MismatchedSchema {}

impl fmt::Display for MismatchedSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Schema V{} in the Catalyst Event Database does not match schema V{} supported by the Crate.",
            self.was,
            self.expected
        )?;
        Ok(())
    }
}

/// Trait to check the schema version of a connection.
#[async_trait]
pub trait SchemaVersion {
    /// Check the schema version.
    /// return the current schema version if its current.
    /// Otherwise return an error.
    async fn schema_version_check(&self) -> Result<u32, Box<dyn Error + Send + Sync + 'static>>;
}

#[async_trait]
impl SchemaVersion for EventDB {
    async fn schema_version_check(&self) -> Result<u32, Box<dyn Error + Send + Sync + 'static>> {
        let conn = self.pool.get().await?;

        let schema_check = conn
            .query_one("SELECT MAX(version) from refinery_schema_history;", &[])
            .await?;

        let current_ver = schema_check.try_get::<usize, u32>(0)?;

        if current_ver == DATABASE_SCHEMA_VERSION {
            Ok(current_ver)
        } else {
            Err(Box::new(MismatchedSchema {
                was: current_ver,
                expected: DATABASE_SCHEMA_VERSION,
            }))
        }
    }
}
