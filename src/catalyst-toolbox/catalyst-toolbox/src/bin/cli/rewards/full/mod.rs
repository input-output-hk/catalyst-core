use std::{fs::File, path::Path};

use color_eyre::Result;
use config::*;
use log::info;
use serde_json::from_reader;

mod config;
mod proposers;
mod python;

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
                proposer_script_path,
                active_voteplans,
                challenges,
                proposals_path,
            },
        outputs:
            Outputs {
                veterans_rewards_output,
                ca_rewards_output,
                proposer_rewards_output,
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
                proposer_stake_threshold,
                proposer_approval_threshold,
            },
    } = config;

    info!("calculating voter rewards");
    super::voters::voter_rewards(
        &block_file,
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

    info!("calculating proposer rewards");
    proposers::proposers_rewards(
        &proposer_script_path,
        &block_file,
        &proposer_rewards_output,
        proposer_stake_threshold,
        proposer_approval_threshold,
        &proposals_path,
        &active_voteplans,
        &challenges,
    )?;

    Ok(())
}
