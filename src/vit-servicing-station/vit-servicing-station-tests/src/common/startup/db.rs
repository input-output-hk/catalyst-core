use assert_fs::{fixture::PathChild, TempDir};
use diesel::{connection::Connection, prelude::*};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::models::{
    api_tokens::APITokenData, challenges::Challenge, funds::Fund, proposals::Proposal,
};

use crate::common::{
    data::Snapshot,
    db::{DbInserter, DbInserterError},
    paths::MIGRATION_DIR,
};
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

pub struct DbBuilder {
    migrations_folder: Option<PathBuf>,
    tokens: Option<Vec<APITokenData>>,
    proposals: Option<Vec<FullProposalInfo>>,
    funds: Option<Vec<Fund>>,
    challenges: Option<Vec<Challenge>>,
}

impl DbBuilder {
    pub fn new() -> Self {
        Self {
            migrations_folder: Some(PathBuf::from_str(MIGRATION_DIR).unwrap()),
            tokens: None,
            proposals: None,
            funds: None,
            challenges: None,
        }
    }

    pub fn with_tokens(&mut self, tokens: Vec<APITokenData>) -> &mut Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn with_token(&mut self, token: APITokenData) -> &mut Self {
        self.with_tokens(vec![token]);
        self
    }

    pub fn with_proposals(&mut self, proposals: Vec<FullProposalInfo>) -> &mut Self {
        self.proposals = Some(proposals);
        self
    }

    pub fn with_challenges(&mut self, challenges: Vec<Challenge>) -> &mut Self {
        self.challenges = Some(challenges);
        self
    }

    pub fn with_snapshot(&mut self, snapshot: &Snapshot) -> &mut Self {
        self.with_proposals(snapshot.proposals());
        self.with_tokens(snapshot.tokens().values().cloned().collect());
        self.with_funds(snapshot.funds());
        self.with_challenges(snapshot.challenges());
        self
    }

    pub fn with_funds(&mut self, funds: Vec<Fund>) -> &mut Self {
        self.funds = Some(funds);
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

    fn try_insert_tokens(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(tokens) = &self.tokens {
            DbInserter::new(connection).insert_tokens(tokens)?;
        }
        Ok(())
    }

    fn try_insert_funds(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(funds) = &self.funds {
            DbInserter::new(connection).insert_funds(funds)?;
        }
        Ok(())
    }

    fn try_insert_proposals(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(proposals) = &self.proposals {
            DbInserter::new(connection).insert_proposals(proposals)?;
        }
        Ok(())
    }

    fn try_insert_challenges(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(challenges) = &self.challenges {
            DbInserter::new(connection).insert_challenges(challenges)?;
        }
        Ok(())
    }

    pub fn build(&self, temp_dir: &TempDir) -> Result<PathBuf, DbBuilderError> {
        let db = temp_dir.child("vit_station.db");
        let db_path = db
            .path()
            .to_str()
            .ok_or(DbBuilderError::CannotExtractTempPath)?;
        println!("Building db in {:?}...", db_path);

        let connection = SqliteConnection::establish(db_path)?;
        self.try_do_migration(&connection)?;
        self.try_insert_tokens(&connection)?;
        self.try_insert_funds(&connection)?;
        self.try_insert_proposals(&connection)?;
        self.try_insert_challenges(&connection)?;
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
    CannotCreateDatabase(#[from] diesel::ConnectionError),
    #[error("Cannot initialize on temp directory")]
    CannotExtractTempPath,
    #[error("migration errors")]
    MigrationsError(#[from] diesel::migration::RunMigrationsError),
}
