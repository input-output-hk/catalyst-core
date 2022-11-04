use crate::db::DbConnection;

pub fn initialize_db_with_migration(db_conn: &DbConnection) {
    // Discard all output
    let mut sink = std::io::sink();

    diesel_migrations::run_pending_migrations_in_directory(
        db_conn,
        &std::path::PathBuf::from("./migrations/sqlite"),
        &mut sink,
    )
    .unwrap();
}
