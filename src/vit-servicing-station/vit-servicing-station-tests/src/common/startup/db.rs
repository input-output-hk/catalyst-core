use assert_fs::{fixture::PathChild, TempDir};
use diesel::{connection::Connection, prelude::*};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::models::{api_tokens::APITokenData, proposals::Proposal};

use crate::common::{
    db::{DbInserter, DbInserterError},
    paths::MIGRATION_DIR,
};

pub struct DbBuilder {
    migrations_folder: Option<PathBuf>,
    token: Option<APITokenData>,
    proposals: Option<Vec<Proposal>>,
}

impl DbBuilder {
    pub fn new() -> Self {
        Self {
            migrations_folder: Some(PathBuf::from_str(MIGRATION_DIR).unwrap()),
            token: None,
            proposals: None,
        }
    }

    pub fn with_token(&mut self, token: APITokenData) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn with_proposals(&mut self, proposals: Vec<Proposal>) -> &mut Self {
        self.proposals = Some(proposals);
        self
    }

    pub fn disable_migrations(&mut self) -> &mut Self {
        self.migrations_folder = None;
        self
    }

    pub fn with_migrations_from<P: AsRef<Path>>(&mut self, migrations_folder: P) -> &mut Self {
        self.migrations_folder = Some(migrations_folder.as_ref().into());
        self
    }

    fn create_db_if_not_exists(&self, db_path: &str) -> Result<(), DbBuilderError> {
        rusqlite::Connection::open(db_path).map_err(DbBuilderError::CannotCreateDatabase)?;
        Ok(())
    }

    fn do_migration(
        &self,
        connection: &SqliteConnection,
        migration_folder: &PathBuf,
    ) -> Result<(), DbBuilderError> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        diesel_migrations::run_pending_migrations_in_directory(
            connection,
            migration_folder,
            &mut handle,
        )
        .map_err(DbBuilderError::MigrationsError)
    }

    fn try_do_migration(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(migrations_folder) = &self.migrations_folder {
            self.do_migration(&connection, migrations_folder)?;
        }
        Ok(())
    }

    fn try_insert_token(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(token) = &self.token {
            DbInserter::new(connection).insert_token(token)?;
        }
        Ok(())
    }

    fn try_insert_proposals(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(proposals) = &self.proposals {
            DbInserter::new(connection).insert_proposals(proposals)?;
        }
        Ok(())
    }

    pub fn build(&self, temp_dir: &TempDir) -> Result<PathBuf, DbBuilderError> {
        let db = temp_dir.child("vit_station.db");
        let db_path = db
            .path()
            .to_str()
            .ok_or_else(|| DbBuilderError::CannotExtractTempPath)?;
        println!("Building db in {:?}...", db_path);

        self.create_db_if_not_exists(db_path)?;

        let connection = SqliteConnection::establish(db_path).unwrap();
        self.try_do_migration(&connection)?;
        self.try_insert_token(&connection)?;
        self.try_insert_proposals(&connection)?;
        Ok(PathBuf::from(db.path()))
    }
}

impl Default for DbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
pub enum DbBuilderError {
    #[error("cannot insert data")]
    DbInserterError(#[from] DbInserterError),
    #[error("Cannot open or create database")]
    CannotCreateDatabase(#[from] rusqlite::Error),
    #[error("Cannot initialize on temp directory")]
    CannotExtractTempPath,
    #[error("migration errors")]
    MigrationsError(#[from] diesel::migration::RunMigrationsError),
}
