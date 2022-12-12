use microtype::microtype;
use serde::Deserialize;

microtype! {
    #[derive(Debug, PartialEq,Eq, Clone)]
    #[string]
    pub String {
        /// Database name
        DbName,
        /// Database user
        DbUser,
        /// Database host
        DbHost,
    }

    #[secret]
    #[string]
    pub String {
         /// Database password
        DbPass,
    }
}

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
