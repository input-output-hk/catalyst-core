use std::{
    fs::File,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use log::info;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::from_reader;

use super::community_advisors::{FundSettingOpt, ProposalRewardsSlotsOpt};

pub(super) fn full_rewards(path: &Path) -> Result<()> {
    let config = from_reader(File::open(path)?)?;
    let Config {
        inputs:
            Inputs {
                block_file,
                vote_count_path,
                snapshot_path,
                reviews_csv,
                assessments_path,
                proposal_bonus_output,
                approved_proposals_path,
            },
        outputs:
            Outputs {
                veterans_rewards_output,
                ca_rewards_output,
            },
        params:
            Params {
                registration_threshold,
                vote_threshold,
                total_rewards,
                rewards_agreement_rate_cutoffs,
                rewards_agreement_rate_modifiers,
                reputation_agreement_rate_cutoffs,
                reputation_agreement_rate_modifiers,
                min_rankings,
                max_rankings_reputation,
                max_rankings_rewards,
                rewards_slots,
                fund_settings,
                ca_seed,
            },
    } = config;

    info!("calculating voter rewards");
    super::voters::voter_rewards(
        Some(block_file),
        vote_count_path,
        snapshot_path,
        registration_threshold,
        vote_threshold,
        total_rewards,
    )?;

    info!("calculating vca rewards");
    super::veterans::vca_rewards(
        reviews_csv,
        veterans_rewards_output,
        rewards_agreement_rate_cutoffs,
        rewards_agreement_rate_modifiers,
        reputation_agreement_rate_cutoffs,
        reputation_agreement_rate_modifiers,
        total_rewards.into(),
        min_rankings,
        max_rankings_reputation,
        max_rankings_rewards,
    )?;

    info!("calculating ca rewards");
    super::community_advisors::ca_rewards(
        assessments_path,
        approved_proposals_path,
        fund_settings,
        rewards_slots,
        ca_rewards_output,
        ca_seed,
        proposal_bonus_output,
    )?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Config {
    inputs: Inputs,
    outputs: Outputs,
    params: Params,
}

#[derive(Debug, Deserialize)]
struct Inputs {
    block_file: PathBuf,
    snapshot_path: PathBuf,
    vote_count_path: PathBuf,
    reviews_csv: PathBuf,
    assessments_path: PathBuf, // is assessments the same as reviews?
    proposal_bonus_output: Option<PathBuf>,
    approved_proposals_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Outputs {
    veterans_rewards_output: PathBuf,
    ca_rewards_output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Params {
    registration_threshold: u64,
    vote_threshold: u64,
    total_rewards: u64,
    rewards_agreement_rate_cutoffs: Vec<Decimal>,
    rewards_agreement_rate_modifiers: Vec<Decimal>,
    reputation_agreement_rate_cutoffs: Vec<Decimal>,
    reputation_agreement_rate_modifiers: Vec<Decimal>,
    min_rankings: usize,
    max_rankings_reputation: usize,
    max_rankings_rewards: usize,
    rewards_slots: ProposalRewardsSlotsOpt,
    fund_settings: FundSettingOpt,
    ca_seed: String,
}
