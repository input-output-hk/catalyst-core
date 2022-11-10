use tempfile::TempDir;

use crate::db::DbConnection;

const PG_INITIAL_SETUP_UP: &str =
    include_str!("../../migrations/postgres/00000000000000_diesel_initial_setup/up.sql");
const PG_SETUP_UP: &str =
    include_str!("../../migrations/postgres/2022-11-03-210841_setup_db/up.sql");
const SQLITE_SETUP_UP: &str =
    include_str!("../../migrations/sqlite/2020-05-22-112032_setup_db/up.sql");

pub fn initialize_db_with_migration(db_conn: &DbConnection) {
    // Discard all output
    let mut sink = std::io::sink();

    match db_conn {
        DbConnection::Sqlite(db_conn) => {
            let t = TempDir::new().unwrap();

            write_tmp_migration(&t, "2020-05-22-112032_setup_db", SQLITE_SETUP_UP);

            diesel_migrations::run_pending_migrations_in_directory(db_conn, t.path(), &mut sink)
        }
        DbConnection::Postgres(db_conn) => {
            let t = TempDir::new().unwrap();

            write_tmp_migration(
                &t,
                "00000000000000_diesel_initial_setup",
                PG_INITIAL_SETUP_UP,
            );
            write_tmp_migration(&t, "2022-11-03-210841_setup_db", PG_SETUP_UP);

            diesel_migrations::run_pending_migrations_in_directory(db_conn, t.path(), &mut sink)
        }
    }
    .unwrap();
}

fn write_tmp_migration(dir: &TempDir, migration_name: &str, up_contents: &str) {
    let migration_dir = dir.path().join(migration_name);

    std::fs::create_dir(migration_dir.as_path()).unwrap();
    std::fs::write(migration_dir.as_path().join("up.sql"), up_contents).unwrap();
    std::fs::write(migration_dir.join("down.sql"), "").unwrap();
}
