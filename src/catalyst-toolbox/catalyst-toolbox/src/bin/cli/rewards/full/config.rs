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
    pub(super) assessments_path: PathBuf,
    pub(super) proposal_bonus_output: Option<PathBuf>,
    pub(super) approved_proposals_path: PathBuf,
    pub(super) active_voteplans: PathBuf,
    pub(super) challenges: PathBuf,
    pub(super) proposals_path: PathBuf,
    pub(super) committee_keys: PathBuf,
    pub(super) excluded_proposals: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Outputs {
    pub(super) voter_rewards_output: PathBuf,
    pub(super) veterans_rewards_output: PathBuf,
    pub(super) ca_rewards_output: PathBuf,
    pub(super) proposer_rewards_output: PathBuf,
}

#[derive(Debug, Deserialize)]
pub(super) struct Params {
    pub(super) voter_params: VoterParams,
    pub(super) proposer_params: ProposerParams,
    pub(super) ca_params: CaParams,
    pub(super) vca_params: VcaParams,
}

#[derive(Debug, Deserialize)]
pub(super) struct VoterParams {
    pub(super) total_rewards: u64,
    pub(super) vote_threshold: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct ProposerParams {
    pub(super) stake_threshold: f64,
    pub(super) approval_threshold: f64,
}

#[derive(Debug, Deserialize)]
pub(super) struct CaParams {
    pub(super) rewards_slots: ProposalRewardsSlotsOpt,
    pub(super) fund_settings: FundSettingOpt,
    pub(super) seed: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct VcaParams {
    pub(super) total_rewards: u64,
    pub(super) rewards_agreement_rate_cutoffs: Vec<Decimal>,
    pub(super) rewards_agreement_rate_modifiers: Vec<Decimal>,
    pub(super) reputation_agreement_rate_cutoffs: Vec<Decimal>,
    pub(super) reputation_agreement_rate_modifiers: Vec<Decimal>,
    pub(super) min_rankings: usize,
    pub(super) max_rankings_reputation: usize,
    pub(super) max_rankings_rewards: usize,
}
