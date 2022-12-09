//! Check if the schema is up-to-date.

use std::{error::Error, collections::HashMap};

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::{pg::Pg, migration::MigrationSource};

// MIGRATIONS is internal only.  The only purpose is to check the schema version in the db.
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Check iof all the migrations we know about are applied, and ONLY those
/// migrations are applied. This will NOT apply any migrations.  Migrations
/// should be applied manually if they are not upo-to-date as there is no single
/// master of the database.  The purpose of this check is to ensure the election
/// db library matches the deployed db schema.
fn all_migrations_synced<S: MigrationSource<Pg>>(
    harness: &mut impl MigrationHarness<Pg>,
    source: S,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // All migrations which were applied but that we have no knowledge of.
    let mut unknown_applied_migrations = Vec::new();

    // Get all currently applied migrations in the DB
    let applied_versions = harness.applied_migrations()?;

    // Get all the migrations that we know about.
    let mut migrations = source
        .migrations()?
        .into_iter()
        .map(|m| (m.name().version().as_owned(), m))
        .collect::<HashMap<_, _>>();

    // Remove all applied migrations from the list of migrations we should have run.
    for applied_version in applied_versions {
        if migrations.remove(&applied_version).is_none() {
            // A migration that should have been applied was not, add it to the error.
            unknown_applied_migrations.push(applied_version);
        }
    }

    // Get the list of migrations we should need to apply.
    let migrations = migrations
        .into_iter()
        .map(|(_, m)| m)
        .collect::<Vec<_>>();

    if unknown_applied_migrations.is_empty() && migrations.is_empty() {
        Ok(())
    } else {
        Ok(())
    }
}

/// Check if the DB has all migrations applied like we expect it should.
pub fn db_version_check(connection: &mut impl MigrationHarness<Pg>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    all_migrations_synced(connection, MIGRATIONS)
}
