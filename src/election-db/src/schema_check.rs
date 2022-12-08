//! Check if the schema is up-to-date.

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

// MIGRATIONS is internal only.  The only purpose is to check the schema version in the db.
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

