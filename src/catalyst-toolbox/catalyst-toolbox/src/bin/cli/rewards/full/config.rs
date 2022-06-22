use std::path::PathBuf;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::cli::rewards::community_advisors::{FundSettingOpt, ProposalRewardsSlotsOpt};

#[derive(Debug, Deserialize)]
pub(super) struct Config {
    pub(super) inputs: Inputs,
    pub(super) outputs: Outputs,
    pub(super) params: Params,
}

#[derive(Debug, Deserialize)]
pub(super) struct Inputs {
    pub(super) block_file: PathBuf,
    pub(super) snapshot_path: PathBuf,
    pub(super) vote_count_path: PathBuf,
    pub(super) reviews_csv: PathBuf,
    pub(super) assessments_path: PathBuf, // is assessments the same as reviews?
    pub(super) proposal_bonus_output: Option<PathBuf>,
    pub(super) approved_proposals_path: PathBuf,
    pub(super) proposer_script_path: PathBuf,
    pub(super) active_voteplans: PathBuf,
    pub(super) challenges: PathBuf,
    pub(super) proposals_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub(super) struct Outputs {
    pub(super) veterans_rewards_output: PathBuf,
    pub(super) ca_rewards_output: PathBuf,
    pub(super) proposer_rewards_output: PathBuf,
}

#[derive(Debug, Deserialize)]
pub(super) struct Params {
    pub(super) registration_threshold: u64,
    pub(super) vote_threshold: u64,
    pub(super) total_rewards: u64,
    pub(super) rewards_agreement_rate_cutoffs: Vec<Decimal>,
    pub(super) rewards_agreement_rate_modifiers: Vec<Decimal>,
    pub(super) reputation_agreement_rate_cutoffs: Vec<Decimal>,
    pub(super) reputation_agreement_rate_modifiers: Vec<Decimal>,
    pub(super) min_rankings: usize,
    pub(super) max_rankings_reputation: usize,
    pub(super) max_rankings_rewards: usize,
    pub(super) rewards_slots: ProposalRewardsSlotsOpt,
    pub(super) fund_settings: FundSettingOpt,
    pub(super) ca_seed: String,
    pub(super) proposer_stake_threshold: f64,
    pub(super) proposer_approval_threshold: f64,
}
