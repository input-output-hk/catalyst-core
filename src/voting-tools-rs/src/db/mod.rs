use crate::data::{DbHost, DbName, DbPass, DbUser};

mod queries;
pub mod types;

// We need to allow this because custom type imports aren't always used in all tables

#[allow(unused_imports)]
#[cfg(test)]
pub mod schema;

#[allow(unused_imports)]
#[cfg(not(test))]
mod schema;

mod utils;

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
    pub password: Option<DbPass>,
}

/// Inner module to hide database internals from database code
mod inner {
    use scheduled_thread_pool::ScheduledThreadPool;
    use std::sync::Arc;

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
    use microtype::secrecy::Zeroize;

    /// Type alias for the connection type provided to diesel code
    pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

    /// A handle to the db-sync database instance
    pub struct Db(Pool<ConnectionManager<PgConnection>>);

    impl Db {
        /// Connect to the database with the provided credentials
        ///
        /// # Errors
        ///
        /// Returns an error if connecting to the database fails
        pub fn connect(
            DbConfig {
                name,
                user,
                host,
                password,
            }: DbConfig,
        ) -> Result<Self> {
            use microtype::secrecy::ExposeSecret;
            let mut password = password
                .map(|p| format!(":{}", p.expose_secret()))
                .unwrap_or_default();

            let url = format!("postgres://{user}{password}@{host}/{name}",);
            let manager = ConnectionManager::new(url);
            let pool = Pool::builder()
                .max_size(50)
                .min_idle(Some(25))
                .thread_pool(Arc::new(
                    ScheduledThreadPool::builder()
                        .num_threads(16)
                        .thread_name_pattern("r2d2-worker-{}")
                        .build(),
                ))
                .build(manager)?;

            password.zeroize();
            Ok(Db(pool))
        }

        /// Execute a query against the database
        pub(crate) fn exec<T, F>(&self, f: F) -> Result<T>
        where
            F: FnOnce(&mut Conn) -> QueryResult<T>,
        {
            let mut conn = self.0.get()?;
            let result = f(&mut conn)?;
            Ok(result)
        }
    }

    impl std::fmt::Debug for Db {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Db")
        }
    }

    /// Types which can be easily used as an ergonomic "query object"
    pub(super) trait DbQuery<'a, T>
    where
        Self: LoadQuery<'a, Conn, T> + QueryFragment<Pg> + Send + 'a,
    {
        fn sql_string(&self) -> String {
            let debug = diesel::debug_query(self);
            format!("{debug}")
        }
    }

    impl<'a, T, S> DbQuery<'a, T> for S where S: LoadQuery<'a, Conn, T> + QueryFragment<Pg> + Send + 'a {}
}
