use microtype::microtype;
use serde::Deserialize;

use crate::cli::NetworkId;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub network_id: NetworkId,
    pub scale: u64,
    pub db: u64,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub name: DbName,
    pub user: DbUser,
    pub host: DbHost,
    pub password: DbPass,
}

microtype! {
    #[derive(Debug, PartialEq)]
    #[string]
    pub String {
        DbName,
        DbUser,
        DbHost,
    }

    #[secret]
    #[string]
    pub String {
        DbPass,
    }
}
