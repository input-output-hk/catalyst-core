use tempfile::TempDir;
use thiserror::Error;

use crate::db::DbConnection;

static PG_MIGRATIONS_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/migrations/postgres");

static SQLITE_MIGRATIONS_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/migrations/sqlite");

#[derive(Debug, Error)]
pub enum InitializeDbWithMigrationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to run migrations: {0}")]
    RunMigration(#[from] diesel_migrations::RunMigrationsError),
}

pub fn initialize_db_with_migration(
    db_conn: &DbConnection,
) -> Result<(), InitializeDbWithMigrationError> {
    let t = TempDir::new()?;

    match &db_conn {
        DbConnection::Sqlite(_) => {
            write_tmp_migration(&t, &SQLITE_MIGRATIONS_DIR)?;
        }
        DbConnection::Postgres(_) => {
            write_tmp_migration(&t, &PG_MIGRATIONS_DIR)?;
        }
    }

    // Discard all output
    let mut sink = std::io::sink();

    crate::q!(
        db_conn,
        diesel_migrations::run_pending_migrations_in_directory(db_conn, t.path(), &mut sink)
    )?;

    Ok(())
}

fn write_tmp_migration(
    temp_dir: &TempDir,
    migrations_dir: &include_dir::Dir<'_>,
) -> std::io::Result<()> {
    for e in migrations_dir.entries() {
        if let include_dir::DirEntry::Dir(d) = e {
            if let Some(dir_name) = d.path().file_name() {
                if let Some(up_file) = d.get_file(d.path().join("up.sql")) {
                    let migration_dir = temp_dir.path().join(dir_name);

                    std::fs::create_dir(migration_dir.as_path())?;
                    std::fs::write(
                        migration_dir.as_path().join("up.sql"),
                        up_file.contents_utf8().unwrap(),
                    )?;
                    std::fs::write(migration_dir.join("down.sql"), "")?;
                }
            }
        }
    }
    Ok(())
}
