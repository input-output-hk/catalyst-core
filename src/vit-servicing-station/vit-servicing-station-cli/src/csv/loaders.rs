use crate::db_utils::{backup_db_file, restore_db_file};
use crate::{db_utils::db_file_exists, task::ExecTask};
use csv::Trim;
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::proposals::{
    community_challenge, simple, ProposalChallengeInfo,
};
use vit_servicing_station_lib::db::{
    load_db_connection_pool, models::challenges::Challenge, models::funds::Fund,
    models::proposals::Proposal, models::voteplans::Voteplan,
};

#[derive(Debug, PartialEq, StructOpt)]
pub enum CSVDataCmd {
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

impl CSVDataCmd {
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

    fn proposals_info_from_csv_proposals(
        proposals: &[super::models::Proposal],
    ) -> io::Result<Vec<ProposalChallengeInfo>> {
        let mut res = Vec::new();
        for proposal in proposals {
            res.push(ProposalChallengeInfo::try_from(proposal.clone())?);
        }
        Ok(res)
    }

    fn handle_load(
        db_url: &str,
        funds_path: &str,
        voteplans_path: &str,
        proposals_path: &str,
        challenges_path: &str,
    ) -> io::Result<()> {
        db_file_exists(db_url)?;
        let funds = CSVDataCmd::load_from_csv::<Fund>(funds_path)?;
        if funds.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Wrong number of input fund in {}, just one fund data can be process at a time",
                    funds_path
                ),
            ));
        }
        let mut voteplans = CSVDataCmd::load_from_csv::<Voteplan>(voteplans_path)?;
        let mut challenges = CSVDataCmd::load_from_csv::<Challenge>(challenges_path)?;
        let csv_proposals = CSVDataCmd::load_from_csv::<super::models::Proposal>(proposals_path)?;
        let mut proposals: Vec<Proposal> = csv_proposals.iter().cloned().map(Into::into).collect();
        let proposals_challenge_info =
            CSVDataCmd::proposals_info_from_csv_proposals(&csv_proposals)?;

        let simple_proposals_data: Vec<simple::ChallengeSqlValues> = proposals_challenge_info
            .iter()
            .zip(proposals.iter())
            .filter_map(|(data, proposal)| match data {
                ProposalChallengeInfo::Simple(res) => {
                    Some(res.to_sql_values_with_proposal_id(&proposal.proposal_id))
                }
                ProposalChallengeInfo::CommunityChallenge(_) => None,
            })
            .collect();

        let community_proposals_data: Vec<community_challenge::ChallengeSqlValues> =
            proposals_challenge_info
                .iter()
                .zip(proposals.iter())
                .filter_map(|(data, proposal)| match data {
                    ProposalChallengeInfo::Simple(_) => None,
                    ProposalChallengeInfo::CommunityChallenge(res) => {
                        Some(res.to_sql_values_with_proposal_id(&proposal.proposal_id))
                    }
                })
                .collect();

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
        for challenge in challenges.iter_mut() {
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
            &challenges,
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
    ) -> io::Result<()> {
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

impl ExecTask for CSVDataCmd {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            CSVDataCmd::Load {
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
