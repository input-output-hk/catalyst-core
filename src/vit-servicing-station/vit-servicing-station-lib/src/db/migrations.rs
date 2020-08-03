use crate::db::DBConnectionPool;

embed_migrations!("../migrations");

pub fn initialize_db_with_migration(pool: &DBConnectionPool) {
    let conn = pool.get().unwrap();
    embedded_migrations::run(&conn).unwrap();
}
