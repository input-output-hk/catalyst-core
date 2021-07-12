use crate::db_utils::{backup_db_file, restore_db_file};
use crate::{db_utils::db_file_exists, task::ExecTask};
use csv::Trim;
use thiserror::Error;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::proposals::{
    community_choice, simple, ProposalChallengeInfo,
};
use vit_servicing_station_lib::db::{
    load_db_connection_pool, models::challenges::Challenge, models::funds::Fund,
    models::proposals::Proposal, models::voteplans::Voteplan,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Wrong number of input fund in {0}, just one fund data can be process at a time")]
    InvalidFundData(String)
}

#[derive(Debug, PartialEq, StructOpt)]
pub enum CsvDataCmd {
    /// Load Funds, Voteplans and Proposals information into a SQLite3 ready file DB.
    Load {
        /// URL of the vit-servicing-station database to interact with
        #[structopt(long = "db-url")]
        db_url: String,

        /// Path to the csv containing funds information
        #[structopt(long = "funds")]
        funds: String,

        /// Path to the csv containing voteplans information
        #[structopt(long = "voteplans")]
        voteplans: String,

        /// Path to the csv containing proposals information
        #[structopt(long = "proposals")]
        proposals: String,

        /// Path to the csv containing challenges information
        #[structopt(long = "challenges")]
        challenges: String,
    },
}

impl CsvDataCmd {
    fn load_from_csv<T: DeserializeOwned>(csv_path: &str) -> io::Result<Vec<T>> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(true)
            .quoting(true)
            .quote(b'"')
            .trim(Trim::All)
            .from_path(csv_path)?;
        let mut results = Vec::new();
        for record in reader.deserialize() {
            match record {
                Ok(data) => {
                    results.push(data);
                }
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Error in file {}.\nCause:\n\t{}", csv_path, e),
                    ))
                }
            }
        }
        Ok(results)
    }

    fn handle_load(
        db_url: &str,
        funds_path: &str,
        voteplans_path: &str,
        proposals_path: &str,
        challenges_path: &str,
    ) -> Result<(), Error> {
        db_file_exists(db_url)?;
        let funds = CsvDataCmd::load_from_csv::<Fund>(funds_path)?;
        if funds.len() != 1 {
            return Err(Error::InvalidFundData(funds_path.to_string()));
        }
        let mut voteplans = CsvDataCmd::load_from_csv::<Voteplan>(voteplans_path)?;
        let mut challenges: HashMap<i32, Challenge> =
            CsvDataCmd::load_from_csv::<Challenge>(challenges_path)?
                .into_iter()
                .map(|c| (c.id, c))
                .collect();
        let csv_proposals = CsvDataCmd::load_from_csv::<super::models::Proposal>(proposals_path)?;
        let mut proposals: Vec<Proposal> = Vec::new();
        let mut simple_proposals_data: Vec<simple::ChallengeSqlValues> = Vec::new();
        let mut community_proposals_data: Vec<community_choice::ChallengeSqlValues> = Vec::new();

        for proposal in csv_proposals {
            let challenge_type = challenges
                .get(&proposal.challenge_id)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Challenge with id {} not found", proposal.challenge_id),
                    )
                })?
                .challenge_type
                .clone();
            let (proposal, challenge_info) =
                proposal.into_db_proposal_and_challenge_info(challenge_type)?;
            match challenge_info {
                ProposalChallengeInfo::Simple(simple) => simple_proposals_data
                    .push(simple.to_sql_values_with_proposal_id(&proposal.proposal_id)),
                ProposalChallengeInfo::CommunityChoice(community_choice) => {
                    community_proposals_data.push(
                        community_choice.to_sql_values_with_proposal_id(&proposal.proposal_id),
                    )
                }
            };
        }

        // start db connection
        let pool = load_db_connection_pool(db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
        let db_conn = pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

        // insert fund and retrieve fund with id
        let fund =
            vit_servicing_station_lib::db::queries::funds::insert_fund(funds[0].clone(), &db_conn)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        // apply fund id in voteplans
        for voteplan in voteplans.iter_mut() {
            voteplan.fund_id = fund.id;
        }

        // apply fund id in proposals
        for proposal in proposals.iter_mut() {
            proposal.fund_id = fund.id;
        }

        // apply fund id to challenges
        for challenge in challenges.values_mut() {
            challenge.fund_id = fund.id;
        }

        vit_servicing_station_lib::db::queries::voteplans::batch_insert_voteplans(
            &voteplans, &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_proposals(
            &proposals, &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_simple_challenge_data(
            &simple_proposals_data,
            &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_community_choice_challenge_data(
            &community_proposals_data,
            &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::challenges::batch_insert_challenges(
            &challenges.values().cloned().collect::<Vec<Challenge>>(),
            &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        Ok(())
    }

    fn handle_load_with_db_backup(
        db_url: &str,
        funds_path: &str,
        voteplans_path: &str,
        proposals_path: &str,
        challenges_path: &str,
    ) -> Result<(), Error> {
        let backup_file = backup_db_file(db_url)?;
        if let Err(e) = Self::handle_load(
            db_url,
            funds_path,
            voteplans_path,
            proposals_path,
            challenges_path,
        ) {
            restore_db_file(backup_file, db_url)?;
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl ExecTask for CsvDataCmd {
    type ResultValue = ();
    type Error = Error;
    fn exec(&self) -> Result<(), Error> {
        match self {
            CsvDataCmd::Load {
                db_url,
                funds,
                voteplans,
                proposals,
                challenges,
            } => Self::handle_load_with_db_backup(db_url, funds, voteplans, proposals, challenges),
        }
    }
}

#[cfg(test)]
mod test {}
