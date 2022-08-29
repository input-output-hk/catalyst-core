use serde::Deserialize;

use crate::model::{DbHost, DbName, DbPass, DbUser, SlotNo, TestnetMagic};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub testnet_magic: Option<TestnetMagic>,
    pub scale: u64,
    pub db: DbConfig,
    pub slot_no: Option<SlotNo>,
}

/// Information required for a database connection
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
