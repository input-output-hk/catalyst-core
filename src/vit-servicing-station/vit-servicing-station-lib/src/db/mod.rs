pub mod migrations;
pub mod models;
pub mod queries;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, PgConnection};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub enum DbConnection {
    Sqlite(PooledConnection<ConnectionManager<SqliteConnection>>),
    Postgres(PooledConnection<ConnectionManager<PgConnection>>),
}

#[derive(Clone)]
pub enum DbConnectionPool {
    Sqlite(Pool<ConnectionManager<SqliteConnection>>),
    Postgres(Pool<ConnectionManager<PgConnection>>),
}

impl DbConnectionPool {
    pub fn get(&self) -> Result<DbConnection, r2d2::Error> {
        match self {
            DbConnectionPool::Sqlite(inner) => inner.get().map(DbConnection::Sqlite),
            DbConnectionPool::Postgres(inner) => inner.get().map(DbConnection::Postgres),
        }
    }
}

#[macro_export]
macro_rules! q {
    ($conn:ident,$query:expr) => {
        match $conn {
            $crate::db::DbConnection::Sqlite($conn) => $query,
            $crate::db::DbConnection::Postgres($conn) => $query,
        }
    };
}

#[macro_export]
macro_rules! execute_q {
    ($conn:ident,$query:expr) => {
        match $conn {
            $crate::db::DbConnection::Sqlite($conn) => diesel::query_dsl::methods::ExecuteDsl::<
                _,
                diesel::sqlite::Sqlite,
            >::execute($query, $conn),

            $crate::db::DbConnection::Postgres($conn) => {
                diesel::query_dsl::methods::ExecuteDsl::<_, diesel::pg::Pg>::execute($query, $conn)
            }
        }
    };
}

// ⚠ WARNING ⚠ : This query is sqlite specific, would need to be changed if backend changes
const SQLITE_TEST_CONN_QUERY: &str = "
SELECT
    name
FROM
    sqlite_master
WHERE
    type ='table' AND
    name NOT LIKE 'sqlite_%';
";

pub fn load_db_connection_pool(db_url: &str) -> Result<DbConnectionPool, Error> {
    if db_url.starts_with("postgres://") {
        Ok(DbConnectionPool::Postgres(
            Pool::builder().build(ConnectionManager::new(db_url))?,
        ))
    } else {
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);
        let pool = Pool::builder().build(manager)?;

        // test db connection or bubble up error
        let conn = pool.get()?;
        conn.execute(SQLITE_TEST_CONN_QUERY)?;

        Ok(DbConnectionPool::Sqlite(pool))
    }
}
