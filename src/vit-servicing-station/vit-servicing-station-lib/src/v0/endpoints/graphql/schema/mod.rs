pub mod challenge;
pub mod funds;
pub mod proposals;
pub mod voteplans;

use crate::db;

pub struct QueryRoot {
    pub db_connection_pool: db::DBConnectionPool,
}
