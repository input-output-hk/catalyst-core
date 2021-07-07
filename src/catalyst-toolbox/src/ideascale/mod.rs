mod fetch;
mod models;

use crate::ideascale::fetch::Scores;
use crate::ideascale::models::de::{Challenge, Fund, Funnel, Proposal};

use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

// TODO: set error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FetchError(#[from] fetch::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    /// Fund number id
    #[structopt(long)]
    fund: usize,

    /// ideascale API token
    #[structopt(long)]
    api_token: String,

    /// Path to a json file containing the related rewards per challenge.
    /// `{ challenge_id:  reward_amount}`
    #[structopt(long)]
    rewards: PathBuf,
}

#[derive(Debug)]
struct IdeaScaleData {
    funnels: HashMap<i32, Funnel>,
    fund: Fund,
    challenges: HashMap<i32, Challenge>,
    proposals: HashMap<i32, Proposal>,
    scores: Scores,
}

pub type Rewards = HashMap<i32, i64>;

async fn fetch_all(fund: usize, api_token: String) -> Result<IdeaScaleData, Error> {
    let funnels_task = tokio::spawn(fetch::get_funnels_data_for_fund(fund, api_token.clone()));
    let funds_task = tokio::spawn(fetch::get_funds_data(api_token.clone()));
    let funnels = funnels_task
        .await??
        .into_iter()
        .map(|f| (f.id, f))
        .collect();
    let funds = funds_task.await??;
    let challenges: Vec<Challenge> = funds
        .iter()
        .flat_map(|f| f.challenges.iter().cloned())
        .collect();
    let proposals_tasks: Vec<_> = challenges
        .iter()
        .map(|c| tokio::spawn(fetch::get_proposals_data(c.id, api_token.clone())))
        .collect();
    let proposals: Vec<Proposal> = futures::future::try_join_all(proposals_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    let stage_ids: HashSet<i32> = proposals.iter().map(|p| p.stage_id).collect();

    let scores_tasks: Vec<_> = stage_ids
        .iter()
        .map(|id| {
            tokio::spawn(fetch::get_assessments_scores_by_stage_id(
                *id,
                api_token.clone(),
            ))
        })
        .collect();

    let scores: Scores = futures::future::try_join_all(scores_tasks)
        .await?
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    Ok(IdeaScaleData {
        funnels,
        fund: funds
            .into_iter()
            .find(|f| f.name.contains(&format!("Fund{}", fund)))
            .unwrap_or_else(|| panic!("Selected fund {}, wasn't among the available funds", fund)),
        challenges: challenges.into_iter().map(|c| (c.id, c)).collect(),
        proposals: proposals.into_iter().map(|p| (p.proposal_id, p)).collect(),
        scores,
    })
}

fn load_json_from_file_path<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let file = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

impl Import {
    fn exec(&self) -> Result<(), Error> {
        let Import {
            fund,
            rewards,
            api_token,
        } = self;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        let idescale_data =
            futures::executor::block_on(runtime.spawn(fetch_all(*fund, api_token.clone())))?
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        let rewards: Rewards = load_json_from_file_path(rewards)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

        Ok(())
    }
}
