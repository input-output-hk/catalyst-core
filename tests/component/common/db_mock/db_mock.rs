use sqlx::{migrate::Migrator, Connection, Executor, PgConnection, PgPool};
///Generic mock database struct
use std::path::Path;
use std::thread;
use tokio::runtime::Runtime;
use uuid::Uuid;

//ToDO fix comments add logs, add errors

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub name: String,
    pub migrations_path: String,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "password".to_string(),
            name: "db_mock_test".to_string(),
            migrations_path: "component/common/db_mock/migrations".to_string(),
        }
    }
}

impl DatabaseSettings {
    ///Return connection string to the database
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    ///Return connection string to postgres instance
    pub fn connection_string_without_db_name(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }

    pub fn get_db_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug)]
pub struct DbMock {
    connection_pool: PgPool,
    settings: DatabaseSettings,
    persist: bool,
}

impl DbMock {
    ///Create and migrate a new database using database settings
    ///and migrations folder
    pub async fn new(settings: DatabaseSettings) -> Self {
        let db_name = settings.get_db_name();
        let host_url = settings.connection_string_without_db_name();
        let db_url = settings.connection_string();
        let migrations = settings.migrations_path.clone();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                //create db
                println!("INFO: Starting database {}\n", &db_name);
                let mut conn = PgConnection::connect(&host_url)
                    .await
                    .expect(&format!("Connection to {} failed", &host_url));
                conn.execute(format!(r#"CREATE DATABASE "{db_name}""#).as_str())
                    .await
                    .expect("Database creation failed");
                //migrate
                println!(
                    "INFO: Migrating database {} with migrations: {}\n",
                    &db_name, &migrations
                );
                let mut conn = PgConnection::connect(&db_url)
                    .await
                    .expect("Database connection failed");
                let migrator = Migrator::new(Path::new(&migrations)).await.unwrap();
                migrator.run(&mut conn).await.expect("Migration failed");
            });
        })
        .join()
        .expect("failed to create database");

        let connection_pool = PgPool::connect(&settings.connection_string())
            .await
            .unwrap();

        Self {
            connection_pool,
            settings,
            persist: false,
        }
    }

    ///Create and migrate a new database using default settings and random generated database name
    pub async fn new_with_random_name(
        mut settings: DatabaseSettings,
        prefix: Option<String>,
    ) -> Self {
        let mut name = Uuid::new_v4().to_string();
        if let Some(prefix) = prefix {
            name = prefix + &name
        };
        settings.name = name;
        DbMock::new(settings).await
    }

    ///Connect to an existing database
    pub async fn connect(settings: DatabaseSettings) -> Self {
        let connection_pool = PgPool::connect(&settings.connection_string())
            .await
            .unwrap();
        Self {
            connection_pool,
            settings,
            persist: false,
        }
    }

    ///Get a pool to the database
    pub async fn get_pool(&self) -> PgPool {
        self.connection_pool.clone()
    }

    pub fn get_settings(&self) -> DatabaseSettings {
        self.settings.clone()
    }

    ///Persist the database
    pub fn persist(&mut self) {
        self.persist = true;
    }
}

impl Drop for DbMock {
    fn drop(&mut self) {
        if !self.persist {
            let host_url = self.settings.connection_string_without_db_name();
            let db_name = self.settings.get_db_name();
            thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                    let mut conn = PgConnection::connect(&host_url).await.unwrap();
                    //terminate existing connections
                    println!("INFO: Dropping database {}\n", &db_name);
                    sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{db_name}'"#))
                    .execute( &mut conn)
                    .await
                    .expect("Terminate all other connections");
                    conn.execute(format!(r#"DROP DATABASE "{db_name}""#).as_str())
                        .await
                        .expect("Error while querying the drop database");
                });
            })
            .join()
            .expect("failed to drop database");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::db_mock::{DatabaseSettings, DbMock};

    #[tokio::test]
    async fn create_migrate_drop_new_db() {
        let _db_mock = DbMock::new(DatabaseSettings::default()).await;
    }
}
