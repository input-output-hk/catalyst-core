pub mod funds;
pub mod proposals;
pub mod vote_options;
pub mod voteplans;

use crate::db;
use std::sync::Arc;

pub struct QueryRoot {
    pub db_connection_pool: Arc<db::DBConnectionPool>,
}
