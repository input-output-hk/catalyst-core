use crate::common::db_mock::DatabaseSettings;
use crate::common::db_mock::DbMock;
use sqlx::PgPool;

pub struct EventDbSettings {
    db_settings_instance: DatabaseSettings,
}

impl EventDbSettings {
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

impl Default for EventDbSettings {
    fn default() -> Self {
        let mut db_settings_instance = DatabaseSettings::default();
        db_settings_instance.migrations_path =
            "component/common/event_db_mock/migrations".to_string();
        db_settings_instance.name = "event_db_test".to_string();
        Self {
            db_settings_instance,
        }
    }
}

pub struct EventDbMock {
    mock_db_instance: DbMock,
}

impl EventDbMock {
    ///Create and migrate a new database using database settings
    pub async fn new(settings: EventDbSettings) -> Self {
        EventDbMock {
            mock_db_instance: DbMock::new(settings.db_settings_instance).await,
        }
    }

    pub async fn new_with_default() -> Self {
        let settings = EventDbSettings::default();
        EventDbMock {
            mock_db_instance: DbMock::new(settings.db_settings_instance).await,
        }
    }

    pub async fn new_with_random_name() -> Self {
        let settings = EventDbSettings::default();
        EventDbMock {
            mock_db_instance: DbMock::new_with_random_name(
                settings.db_settings_instance,
                Some("event_db_test".to_string()),
            )
            .await,
        }
    }

    ///Connect to an existing database
    pub async fn connect(settings: EventDbSettings) -> Self {
        Self {
            mock_db_instance: DbMock::connect(settings.db_settings_instance).await,
        }
    }

    ///Connect to default database
    pub async fn connect_to_default() -> Self {
        EventDbMock::connect(EventDbSettings::default()).await
    }

    ///Get a pool to the database
    pub async fn get_pool(&self) -> PgPool {
        self.mock_db_instance.get_pool().await
    }

    pub fn get_settings(&self) -> EventDbSettings {
        EventDbSettings {
            db_settings_instance: self.mock_db_instance.get_settings(),
        }
    }

    ///Persist the database
    pub fn persist(&mut self) {
        self.mock_db_instance.persist();
    }

    ///Insert new event with event_id and not nullable fields in the event table
    pub async fn insert_event(&self, event_id: i32) {
        let event_name = format!("event_test_{}", event_id);
        sqlx::query!(r#"INSERT INTO event (row_id, name, description, committee_size, committee_threshold) VALUES($1, $2, 'test_description', 1, 1)"#, event_id,event_name)
        .execute(&self.get_pool().await)
        .await
        .expect("Failed to insert event id into event database");
    }

    ///Get event with event_id from event db database
    /// TODO return event struct
    pub async fn get_event(&self, event_id: i32) {
        sqlx::query!(r#"SELECT * FROM event WHERE row_id = $1"#, event_id)
            .fetch_one(&self.get_pool().await)
            .await
            .expect("Failed to get event from event database");
    }
}

impl Drop for EventDbMock {
    fn drop(&mut self) {
        drop(&mut self.mock_db_instance);
    }
}

#[cfg(test)]
mod tests {
    use crate::common::event_db_mock::EventDbMock;

    #[tokio::test]
    async fn create_and_drop_new_db() {
        let event_db = EventDbMock::new_with_random_name().await;
        event_db.insert_event(1).await;
        let pool = event_db.get_pool().await;
        let (id, name) = sqlx::query_as::<_, (i32, String)>("SELECT row_id, name FROM event")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(id, 1);
        assert_eq!(name, "event_test_1");
    }
}
