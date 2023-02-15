use diesel::RunQueryDsl;
use rand::Rng;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData, challenges::Challenge, funds::Fund,
};
use vit_servicing_station_lib::db::DbConnection;

use crate::common::{
    data::Snapshot,
    db::{DbInserter, DbInserterError},
};
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

pub struct DbBuilder {
    tokens: Option<Vec<ApiTokenData>>,
    proposals: Option<Vec<FullProposalInfo>>,
    funds: Option<Vec<Fund>>,
    challenges: Option<Vec<Challenge>>,
    advisor_reviews: Option<Vec<AdvisorReview>>,
    groups: Option<Vec<Group>>,
}

impl DbBuilder {
    pub fn new() -> Self {
        Self {
            tokens: None,
            proposals: None,
            funds: None,
            challenges: None,
            advisor_reviews: None,
            groups: None,
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
        self.with_groups(snapshot.groups());
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

    pub fn with_groups(&mut self, groups: Vec<Group>) -> &mut Self {
        self.groups = Some(groups);
        self
    }

    fn try_insert_tokens(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(tokens) = &self.tokens {
            DbInserter::new(connection).insert_tokens(tokens)?;
        }
        Ok(())
    }

    fn try_insert_funds(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(funds) = &self.funds {
            DbInserter::new(connection).insert_funds(funds)?;
        }
        Ok(())
    }

    fn try_insert_proposals(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(proposals) = &self.proposals {
            DbInserter::new(connection).insert_proposals(proposals)?;
        }
        Ok(())
    }

    fn try_insert_challenges(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(challenges) = &self.challenges {
            DbInserter::new(connection).insert_challenges(challenges)?;
        }

        Ok(())
    }

    fn try_insert_reviews(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(reviews) = &self.advisor_reviews {
            DbInserter::new(connection).insert_advisor_reviews(reviews)?;
        }
        Ok(())
    }

    fn try_insert_groups(&self, connection: &DbConnection) -> Result<(), DbBuilderError> {
        if let Some(groups) = &self.groups {
            DbInserter::new(connection).insert_groups(groups)?;
        }
        Ok(())
    }

    pub fn build(&self) -> Result<String, DbBuilderError> {
        let db_url = match std::env::var("TEST_DATABASE_URL") {
            Ok(u) => u,
            Err(std::env::VarError::NotPresent) => {
                return Err(DbBuilderError::MissingDatabaseUrlEnvVar)
            }
            Err(e) => return Err(DbBuilderError::DatabaseUrlEnvVar(e)),
        };

        let (db_url, connection) = {
            let pool = vit_servicing_station_lib::db::load_db_connection_pool(&db_url).unwrap();
            let connection = pool.get().unwrap();

            // create a new database to use when testing
            let tmp_db_name = rand::thread_rng()
                .sample_iter(rand::distributions::Alphanumeric)
                .filter(char::is_ascii_alphabetic)
                .take(8)
                .collect::<String>()
                .to_lowercase();

            diesel::sql_query(format!("CREATE DATABASE {}", tmp_db_name))
                .execute(&connection)
                .unwrap();

            // reconnect to the created database
            let db_url = format!("{}/{}", db_url, tmp_db_name);
            let pool = vit_servicing_station_lib::db::load_db_connection_pool(&db_url).unwrap();

            (db_url, pool.get().unwrap())
        };

        vit_servicing_station_lib::db::migrations::initialize_db_with_migration(&connection)?;
        self.try_insert_tokens(&connection)?;
        self.try_insert_funds(&connection)?;
        self.try_insert_groups(&connection)?;
        self.try_insert_challenges(&connection)?;
        self.try_insert_proposals(&connection)?;
        self.try_insert_reviews(&connection)?;

        Ok(db_url)
    }

    pub fn build_into_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, DbBuilderError> {
        let path = path.as_ref();
        let db_path = path.to_str().ok_or(DbBuilderError::CannotExtractTempPath)?;

        let pool = vit_servicing_station_lib::db::load_db_connection_pool(db_path).unwrap();
        let connection = pool.get().unwrap();

        vit_servicing_station_lib::db::migrations::initialize_db_with_migration(&connection)?;
        self.try_insert_tokens(&connection)?;
        self.try_insert_funds(&connection)?;
        self.try_insert_groups(&connection)?;
        self.try_insert_challenges(&connection)?;
        self.try_insert_proposals(&connection)?;
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
    #[error("missing TEST_DATABASE_URL env var")]
    MissingDatabaseUrlEnvVar,
    #[error("failed to read database url env var")]
    DatabaseUrlEnvVar(#[from] std::env::VarError),
    #[error("cannot insert data")]
    DbInserterError(#[from] DbInserterError),
    #[error("Cannot open or create database")]
    CannotCreateDatabase(#[from] diesel::ConnectionError),
    #[error("Cannot initialize on temp directory")]
    CannotExtractTempPath,
    #[error("migration errors")]
    MigrationsError(#[from] diesel_migrations::RunMigrationsError),
}
