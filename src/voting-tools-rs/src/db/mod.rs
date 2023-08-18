use crate::data::{DbHost, DbName, DbPass, DbUser};

pub mod queries;

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
    /// The time limit in seconds applied to each socket-level connection attempt.
    pub connect_timeout: u32,
    /// The number of seconds of inactivity after which a keepalive message is sent to the server
    pub keepalives_idle: u32,
    /// The time interval between TCP keepalive probes
    pub keepalives_interval: u32,
    /// The maximum number of TCP keepalive probes that will be sent before dropping a connection.
    pub keepalives_retries: u32,
}
