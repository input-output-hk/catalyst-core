use crate::db_utils::{backup_db_file, restore_db_file};
use crate::task::ExecTask;
use clap::Parser;
use csv::Trim;
use diesel::{Insertable, QueryDsl, RunQueryDsl};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};
use thiserror::Error;
use vit_servicing_station_lib::db;
use vit_servicing_station_lib::db::models::goals::InsertGoal;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::models::proposals::{
    community_choice, simple, ProposalChallengeInfo, ProposalVotePlan, ProposalVotePlanCommon,
};
use vit_servicing_station_lib::db::models::vote::Vote;
use vit_servicing_station_lib::db::schema::community_advisors_reviews as community_advisors_reviews_dsl;
use vit_servicing_station_lib::db::{
    load_db_connection_pool,
    models::{proposals::Proposal, voteplans::Voteplan},
};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Invalid Fund Data: {0}")]
    InvalidFundData(String),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
}
#[derive(Debug, Eq, PartialEq, Parser)]
pub struct LoadCmd {
    /// Additional import settings
    #[clap(long = "additional-settings")]
    settings: Option<PathBuf>,

    /// URL of the vit-servicing-station database to interact with
    #[clap(long = "db-url")]
    db_url: String,

    /// Path to the csv containing funds information
    /// At the moment, it's required these are ordered.
    ///
    /// Also the first fund being the current one, which means previous funds should not be
    /// included. This restriction may be lifted in the future.
    #[clap(long = "funds")]
    funds: PathBuf,

    /// Path to the csv containing voteplans information
    #[clap(long = "voteplans")]
    voteplans: PathBuf,

    /// Path to the csv containing proposals information
    #[clap(long = "proposals")]
    proposals: PathBuf,

    /// Path to the csv containing challenges information
    #[clap(long = "challenges")]
    challenges: PathBuf,

    /// Path to the csv containing advisor reviews information
    #[clap(long = "reviews")]
    reviews: PathBuf,

    /// Path to the csv containing goals information
    #[clap(long = "goals")]
    goals: Option<PathBuf>,

    /// Path to the csv containing goals information
    #[clap(long = "groups")]
    groups: Option<PathBuf>,

    /// Path to the folder containing csv with votes information
    #[clap(long = "votes")]
    votes: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq, Parser)]
pub enum CsvDataCmd {
    /// Loads data into db
    Load(LoadCmd),
}

#[derive(Serialize, Deserialize, Default)]
pub struct ImportSettings {
    /// override fund id. This setting is extremely useful in example for fund7 where fund id is still 6 instead of 7
    pub force_fund_id: Option<i32>,
}

impl ExecTask for CsvDataCmd {
    type ResultValue = ();
    type Error = Error;
    fn exec(&self) -> Result<(), Error> {
        match self {
            Self::Load(load) => load.exec(),
        }
    }
}

impl LoadCmd {
    fn load_from_csv<T: DeserializeOwned>(csv_path: &Path) -> io::Result<Vec<T>> {
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
                        format!(
                            "Error in file {}.\nCause:\n\t{}",
                            csv_path.to_string_lossy(),
                            e
                        ),
                    ))
                }
            }
        }
        Ok(results)
    }

    fn handle_load(&self) -> Result<(), Error> {
        let settings = {
            if let Some(path) = &self.settings {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                serde_json::from_reader(reader)?
            } else {
                ImportSettings::default()
            }
        };

        let mut funds = LoadCmd::load_from_csv::<super::models::Fund>(&self.funds)?;

        if let Some(override_fund_id) = settings.force_fund_id {
            funds.iter_mut().for_each(|x| x.id = override_fund_id);
        }

        let mut voteplans: Vec<Voteplan> =
            LoadCmd::load_from_csv::<super::models::Voteplan>(&self.voteplans)?
                .into_iter()
                .map(|x| x.try_into().unwrap())
                .collect();
        let mut challenges = LoadCmd::load_from_csv::<super::models::Challenge>(&self.challenges)?;

        let csv_proposals = LoadCmd::load_from_csv::<super::models::Proposal>(&self.proposals)?;
        let mut reviews = LoadCmd::load_from_csv::<super::models::AdvisorReview>(&self.reviews)?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<db::models::community_advisors_reviews::AdvisorReview>, _>>()
            .unwrap();

        let mut goals: Vec<InsertGoal> = if let Some(goals_path) = &self.goals {
            LoadCmd::load_from_csv::<InsertGoal>(goals_path)?
        } else {
            vec![]
        };

        let mut proposals: Vec<Proposal> = Vec::new();
        let mut simple_proposals_data: Vec<simple::ChallengeSqlValues> = Vec::new();
        let mut community_proposals_data: Vec<community_choice::ChallengeSqlValues> = Vec::new();

        for proposal in csv_proposals.clone() {
            let challenge_type = challenges
                .iter()
                .find(|c| proposal.challenge_id == c.id)
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
            proposals.push(proposal);
        }

        // start db connection
        let pool = load_db_connection_pool(&self.db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
        let db_conn = &pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

        let mut funds_iter = funds.into_iter().map(|x| x.try_into().unwrap());

        // insert fund and retrieve fund with id
        let fund = db::queries::funds::insert_fund(
            funds_iter
                .next()
                .ok_or_else(|| Error::InvalidFundData(self.funds.to_string_lossy().to_string()))?,
            db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        for fund in funds_iter {
            vit_servicing_station_lib::db::queries::funds::insert_fund(fund, db_conn)
                .map_err(|e| io::Error::other(format!("{}", e)))?;
        }

        // apply fund id in voteplans
        for voteplan in voteplans.iter_mut() {
            voteplan.fund_id = fund.id;
        }

        let new_challenge_ids: HashMap<i32, i32> = challenges
            .iter()
            .map(|c| {
                let url = url::Url::from_str(&c.challenge_url).unwrap();
                //format: https://cardano.ideascale.com/c/campaigns/XXXXX/
                let paths_segments = url.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
                let id = paths_segments.get(2).unwrap();
                (c.id, id.parse().unwrap())
            })
            .collect();

        // apply fund id to challenges
        for challenge in challenges.iter_mut() {
            challenge.id = *new_challenge_ids.get(&challenge.id).unwrap();
        }

        // apply fund id in proposals
        for proposal in proposals.iter_mut() {
            proposal.fund_id = fund.id;
            proposal.challenge_id = *new_challenge_ids.get(&proposal.challenge_id).unwrap();
            proposal.internal_id = proposal.proposal_id.parse().unwrap();
        }

        for goal in goals.iter_mut() {
            goal.fund_id = fund.id;
        }

        let advisors = db::schema::community_advisors_reviews::dsl::community_advisors_reviews
            .select(community_advisors_reviews_dsl::id)
            .load::<i32>(db_conn)?;
        let max_id = advisors.iter().max().unwrap_or(&0i32);

        for review in reviews.iter_mut() {
            review.id += max_id;
        }

        vit_servicing_station_lib::db::queries::voteplans::batch_insert_voteplans(
            &voteplans, db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_proposals(
            &proposals, db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        let proposals_voteplans = csv_proposals
            .iter()
            .cloned()
            .map(|proposal| ProposalVotePlan {
                proposal_id: proposal.proposal_id.clone(),
                common: ProposalVotePlanCommon {
                    chain_voteplan_id: proposal.chain_voteplan_id.to_string(),
                    chain_proposal_index: proposal.chain_proposal_index,
                },
            });

        vit_servicing_station_lib::db::queries::proposals::batch_insert_proposals_voteplans(
            proposals_voteplans,
            db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_simple_challenge_data(
            &simple_proposals_data,
            db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_community_choice_challenge_data(
            &community_proposals_data,
            db_conn,
        )
            .map_err(|e| io::Error::other(format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::challenges::batch_insert_challenges(
            &challenges
                .into_iter()
                .map(|c| c.into_db_challenge_values())
                .collect::<Vec<_>>(),
            db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::community_advisors_reviews::batch_insert_advisor_reviews(&reviews, db_conn)
            .map_err(|e| io::Error::other(format!("{}", e)))?;

        let groups = if let Some(groups_path) = &self.groups {
            LoadCmd::load_from_csv::<vit_servicing_station_lib::db::models::groups::Group>(
                groups_path,
            )?
        } else {
            voteplans
                .iter()
                .cloned()
                .map(|v| Group {
                    fund_id: v.fund_id,
                    token_identifier: v.token_identifier,
                    group_id: "direct".to_string(),
                })
                .collect()
        };

        vit_servicing_station_lib::db::queries::groups::batch_insert(
            &groups.into_iter().map(|c| c.values()).collect::<Vec<_>>(),
            db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;

        Ok(())
    }

    fn list_of_csv_paths<P: AsRef<Path>>(root: P) -> io::Result<Vec<PathBuf>> {
        let mut result = vec![];

        for path in fs::read_dir(root)? {
            let path = path?.path();
            if let Some("csv") = path.extension().and_then(OsStr::to_str) {
                result.push(path.to_owned());
            }
        }
        Ok(result)
    }

    fn handle_votes_load(db_url: &str, votes_path: &Path) -> Result<(), Error> {
        let mut votes = vec![];

        for csv_file in Self::list_of_csv_paths(votes_path)? {
            votes.extend(LoadCmd::load_from_csv::<Vote>(&csv_file)?);
        }

        // start db connection
        let pool = load_db_connection_pool(db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
        let db_conn = pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::votes::batch_insert_votes_data(
            &votes.into_iter().map(|c| c.values()).collect::<Vec<_>>(),
            &db_conn,
        )
        .map_err(|e| io::Error::other(format!("{}", e)))?;
        Ok(())
    }

    pub fn exec(&self) -> Result<(), Error> {
        self.handle_load()?;

        if let Some(votes_folder) = &self.votes {
            let backup_file = backup_db_file(&self.db_url)?;
            return if let Err(e) = Self::handle_votes_load(&self.db_url, votes_folder) {
                restore_db_file(backup_file, &self.db_url)?;
                Err(e)
            } else {
                Ok(())
            };
        }
        Ok(())
    }
}
