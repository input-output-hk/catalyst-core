use crate::model::*;

mod queries;
pub mod types;

// We need to allow this because custom type imports aren't always used in all tables
#[allow(unused_imports)]
mod schema;

pub use inner::{Conn, Db};
use serde::Deserialize;

/// Information required to connect to a database
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    /// The name of the database
    pub name: DbName,
    /// The user to connect with
    pub user: DbUser,
    /// The hostname to connect to
    pub host: DbHost,
    /// The corresponding password for this user
    pub password: DbPass,
}

/// Inner module to hide database internals from database code
///
/// Calls to the database are blocking, which blocks tokio. Instead, all calls should go through
/// `exec`, which uses runs them on a thread pool with [`tokio::task::spawn_blocking`].
///
/// This module hides the inner connection pool from queries, so they cannot accidentally get a
/// regular connection and block the executor
mod inner {
    use super::DbConfig;
    use color_eyre::Result;
    use diesel::{
        pg::Pg,
        query_builder::QueryFragment,
        query_dsl::LoadQuery,
        r2d2::{ConnectionManager, Pool, PooledConnection},
        result::QueryResult,
        PgConnection,
    };

    /// Type alias for the connection type provided to diesel code
    pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

    /// A handle to the db-sync database instance
    pub struct Db(Pool<ConnectionManager<PgConnection>>);

    impl Db {
        /// Connect to the database with the provided credentials
        pub async fn connect(
            DbConfig {
                name,
                user,
                host,
                password,
            }: DbConfig,
        ) -> Result<Self> {
            use microtype::secrecy::ExposeSecret;

            let url = format!(
                "postgres://{user}:{password}@{host}/{name}?readOnly=true",
                password = password.expose_secret()
            );
            let manager = ConnectionManager::new(&url);
            let pool = Pool::new(manager)?;

            Ok(Db(pool))
        }

        /// Execute a query against the database
        ///
        /// All queries must go through this function, to force them to go through tokio's
        /// `spawn_blocking` mechanism, to prevent them from blocking the executor
        pub(super) async fn exec<T, F>(&self, f: F) -> Result<T>
        where
            T: Send + 'static,
            F: Send + 'static + FnOnce(&mut Conn) -> QueryResult<T>,
        {
            let mut conn = self.0.get()?;
            let result = tokio::task::spawn_blocking(move || f(&mut conn)).await??;
            Ok(result)
        }
    }

    impl std::fmt::Debug for Db {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Db")
        }
    }

    /// Types which can be easily used as an ergonomic "query object"
    pub(super) trait DbQuery<'a, T>: LoadQuery<'a, Conn, T> + QueryFragment<Pg> {
        fn sql_string(&self) -> String {
            let debug = diesel::debug_query(self);
            format!("{debug}")
        }
    }

    impl<'a, T, S> DbQuery<'a, T> for S where S: LoadQuery<'a, Conn, T> + QueryFragment<Pg> {}
}
