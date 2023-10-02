use assert_fs::{fixture::PathChild, TempDir};
use diesel::{connection::Connection, prelude::*};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData, challenges::Challenge, funds::Fund,
};

use crate::common::{
    data::Snapshot,
    db::{DbInserter, DbInserterError},
    paths::MIGRATION_DIR,
};
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

const VIT_STATION_DB: &str = "vit_station.db";

pub struct DbBuilder {
    migrations_folder: Option<PathBuf>,
    tokens: Option<Vec<ApiTokenData>>,
    proposals: Option<Vec<FullProposalInfo>>,
    funds: Option<Vec<Fund>>,
    challenges: Option<Vec<Challenge>>,
    advisor_reviews: Option<Vec<AdvisorReview>>,
}

impl DbBuilder {
    pub fn new() -> Self {
        Self {
            migrations_folder: Some(PathBuf::from_str(MIGRATION_DIR).unwrap()),
            tokens: None,
            proposals: None,
            funds: None,
            challenges: None,
            advisor_reviews: None,
        }
    }

    pub fn with_tokens(&mut self, tokens: Vec<ApiTokenData>) -> &mut Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn with_token(&mut self, token: ApiTokenData) -> &mut Self {
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
        self.with_advisor_reviews(snapshot.advisor_reviews());
        self
    }

    pub fn with_funds(&mut self, funds: Vec<Fund>) -> &mut Self {
        self.funds = Some(funds);
        self
    }

    pub fn with_advisor_reviews(&mut self, reviews: Vec<AdvisorReview>) -> &mut Self {
        self.advisor_reviews = Some(reviews);
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
        migration_folder: &Path,
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
            self.do_migration(connection, migrations_folder)?;
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

    fn try_insert_reviews(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(reviews) = &self.advisor_reviews {
            DbInserter::new(connection).insert_advisor_reviews(reviews)?;
        }
        Ok(())
    }

    pub fn build(&self, temp_dir: &TempDir) -> Result<PathBuf, DbBuilderError> {
        self.build_into_path(temp_dir.child(VIT_STATION_DB).path())
    }

    pub fn build_into_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, DbBuilderError> {
        let path = path.as_ref();
        let db_path = path.to_str().ok_or(DbBuilderError::CannotExtractTempPath)?;
        println!("Building db in {:?}...", db_path);

        let connection = SqliteConnection::establish(db_path)?;
        self.try_do_migration(&connection)?;
        self.try_insert_tokens(&connection)?;
        self.try_insert_funds(&connection)?;
        self.try_insert_proposals(&connection)?;
        self.try_insert_challenges(&connection)?;
        self.try_insert_reviews(&connection)?;
        Ok(path.to_path_buf())
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
