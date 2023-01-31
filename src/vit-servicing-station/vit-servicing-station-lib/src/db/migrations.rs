use crate::db::DbConnection;

embed_migrations!("./migrations");

pub type InitializeDbWithMigrationError = diesel_migrations::RunMigrationsError;

pub fn initialize_db_with_migration(
    db_conn: &DbConnection,
) -> Result<(), InitializeDbWithMigrationError> {
    embedded_migrations::run(db_conn)
}
