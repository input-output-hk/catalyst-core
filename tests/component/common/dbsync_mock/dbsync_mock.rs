use crate::common::db_mock::DatabaseSettings;
use crate::common::db_mock::DbMock;
use sqlx::PgPool;

pub struct DbSyncSettings {
    db_settings_instance: DatabaseSettings,
}

impl DbSyncSettings {
    ///Return connection string to the database
    pub fn connection_string(&self) -> String {
        self.db_settings_instance.connection_string()
    }

    ///Return connection string to postgres instance
    pub fn connection_string_without_db_name(&self) -> String {
        self.db_settings_instance
            .connection_string_without_db_name()
    }

    pub fn get_db_name(&self) -> String {
        self.db_settings_instance.get_db_name()
    }
}

impl Default for DbSyncSettings {
    fn default() -> Self {
        let mut db_settings_instance = DatabaseSettings::default();
        db_settings_instance.migrations_path =
            "component/common/dbsync_mock/migrations".to_string();
        db_settings_instance.name = "dbsync_test".to_string();
        Self {
            db_settings_instance,
        }
    }
}

pub struct DbSyncMock {
    mock_db_instance: DbMock,
}

impl DbSyncMock {
    ///Create and migrate a new database using database settings
    pub async fn new(settings: DbSyncSettings) -> Self {
        DbSyncMock {
            mock_db_instance: DbMock::new(settings.db_settings_instance).await,
        }
    }

    pub async fn new_with_default() -> Self {
        let settings = DbSyncSettings::default();
        DbSyncMock {
            mock_db_instance: DbMock::new(settings.db_settings_instance).await,
        }
    }

    pub async fn new_with_random_name() -> Self {
        let settings = DbSyncSettings::default();
        DbSyncMock {
            mock_db_instance: DbMock::new_with_random_name(
                settings.db_settings_instance,
                Some("dbsync_test".to_string()),
            )
            .await,
        }
    }

    ///Connect to an existing database
    pub async fn connect(settings: DbSyncSettings) -> Self {
        Self {
            mock_db_instance: DbMock::connect(settings.db_settings_instance).await,
        }
    }

    ///Connect to default database
    pub async fn connect_to_default() -> Self {
        DbSyncMock::connect(DbSyncSettings::default()).await
    }

    ///Get a pool to the database
    pub async fn get_pool(&self) -> PgPool {
        self.mock_db_instance.get_pool().await
    }

    pub fn get_settings(&self) -> DbSyncSettings {
        DbSyncSettings {
            db_settings_instance: self.mock_db_instance.get_settings(),
        }
    }

    ///Persist the database
    pub fn persist(&mut self) {
        self.mock_db_instance.persist();
    }
}

impl Drop for DbSyncMock {
    fn drop(&mut self) {
        drop(&mut self.mock_db_instance);
    }
}

#[cfg(test)]
mod tests {
    use crate::common::dbsync_mock::DbSyncMock;

    #[tokio::test]
    async fn create_and_drop_new_db() {
        let _dbsync = DbSyncMock::new_with_random_name().await;
    }
}
