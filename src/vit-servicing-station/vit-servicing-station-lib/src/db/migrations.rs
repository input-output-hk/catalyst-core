use crate::db::DBConnection;

embed_migrations!("../migrations");

pub fn initialize_db_with_migration(db_conn: &DBConnection) {
    embedded_migrations::run(db_conn).unwrap();
}
