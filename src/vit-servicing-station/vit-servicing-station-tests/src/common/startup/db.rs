use assert_fs::{fixture::PathChild, TempDir};
use diesel::{connection::Connection, prelude::*};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::{
    models::{api_tokens::APITokenData, proposals::Proposal},
    schema::{api_tokens, proposals, voteplans},
};

use crate::common::paths::MIGRATION_DIR;

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
            let values = (
                api_tokens::dsl::token.eq(&(*token.token.as_ref())),
                api_tokens::dsl::creation_time.eq(token.creation_time),
                api_tokens::dsl::expire_time.eq(token.expire_time),
            );

            diesel::insert_into(api_tokens::table)
                .values(values)
                .execute(connection)
                .map_err(DbBuilderError::DieselError)?;
        }
        Ok(())
    }

    fn try_insert_proposals(&self, connection: &SqliteConnection) -> Result<(), DbBuilderError> {
        if let Some(proposals) = &self.proposals {
            for proposal in proposals {
                let values = (
                    proposals::proposal_id.eq(proposal.proposal_id.clone()),
                    proposals::proposal_category
                        .eq(proposal.proposal_category.category_name.clone()),
                    proposals::proposal_title.eq(proposal.proposal_title.clone()),
                    proposals::proposal_summary.eq(proposal.proposal_summary.clone()),
                    proposals::proposal_problem.eq(proposal.proposal_problem.clone()),
                    proposals::proposal_solution.eq(proposal.proposal_solution.clone()),
                    proposals::proposal_public_key.eq(proposal.proposal_public_key.clone()),
                    proposals::proposal_funds.eq(proposal.proposal_funds),
                    proposals::proposal_url.eq(proposal.proposal_url.clone()),
                    proposals::proposal_files_url.eq(proposal.proposal_files_url.clone()),
                    proposals::proposer_name.eq(proposal.proposer.proposer_name.clone()),
                    proposals::proposer_contact.eq(proposal.proposer.proposer_email.clone()),
                    proposals::proposer_url.eq(proposal.proposer.proposer_url.clone()),
                    proposals::chain_proposal_id.eq(proposal.chain_proposal_id.clone()),
                    proposals::chain_proposal_index.eq(proposal.chain_proposal_index),
                    proposals::chain_vote_options.eq(proposal.chain_vote_options.as_csv_string()),
                    proposals::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
                );
                diesel::insert_into(proposals::table)
                    .values(values)
                    .execute(connection)?;

                // insert the related fund voteplan information
                let voteplan_values = (
                    voteplans::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
                    voteplans::chain_vote_start_time.eq(proposal.chain_vote_start_time),
                    voteplans::chain_vote_end_time.eq(proposal.chain_vote_end_time),
                    voteplans::chain_committee_end_time.eq(proposal.chain_committee_end_time),
                    voteplans::chain_voteplan_payload.eq(proposal.chain_voteplan_payload.clone()),
                    voteplans::fund_id.eq(proposal.fund_id),
                );

                diesel::insert_into(voteplans::table)
                    .values(voteplan_values)
                    .execute(connection)?;
            }
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
    #[error("internal diesel error")]
    DieselError(#[from] diesel::result::Error),
    #[error("Cannot open or create database")]
    CannotCreateDatabase(#[from] rusqlite::Error),
    #[error("Cannot initialize on temp directory")]
    CannotExtractTempPath,
    #[error("migration errors")]
    MigrationsError(#[from] diesel::migration::RunMigrationsError),
}
