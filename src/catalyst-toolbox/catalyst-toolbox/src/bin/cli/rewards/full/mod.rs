use catalyst_toolbox::{
    rewards::proposers::{OutputFormat, ProposerRewards},
    utils::json_from_file,
};
use color_eyre::Result;
use config::*;
use std::path::Path;
use tracing::info;

mod config;

pub(super) fn full_rewards(path: &Path) -> Result<()> {
    let config = json_from_file(path)?;
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
                active_voteplans,
                challenges,
                proposals_path,
                committee_keys,
                excluded_proposals,
            },
        outputs:
            Outputs {
                voter_rewards_output,
                veterans_rewards_output,
                ca_rewards_output,
                proposer_rewards_output,
            },
        params:
            Params {
                voter_params,
                proposer_params,
                ca_params,
                vca_params,
            },
    } = config;

    info!("calculating voter rewards");
    super::voters::voter_rewards(
        &Some(voter_rewards_output),
        &vote_count_path,
        &snapshot_path,
        voter_params.vote_threshold,
        voter_params.total_rewards,
    )?;

    info!("calculating vca rewards");
    super::veterans::vca_rewards(
        reviews_csv,
        veterans_rewards_output,
        vca_params.rewards_agreement_rate_cutoffs,
        vca_params.rewards_agreement_rate_modifiers,
        vca_params.reputation_agreement_rate_cutoffs,
        vca_params.reputation_agreement_rate_modifiers,
        vca_params.total_rewards.into(),
        vca_params.min_rankings,
        vca_params.max_rankings_reputation,
        vca_params.max_rankings_rewards,
    )?;

    info!("calculating ca rewards");
    super::community_advisors::ca_rewards(
        assessments_path,
        approved_proposals_path,
        ca_params.fund_settings,
        ca_params.rewards_slots,
        ca_rewards_output,
        ca_params.seed,
        proposal_bonus_output,
    )?;

    info!("calculating proposer rewards");
    super::proposers::rewards(&ProposerRewards {
        output: proposer_rewards_output,
        block0: block_file,
        total_stake_threshold: proposer_params.stake_threshold,
        approval_threshold: proposer_params.approval_threshold,
        proposals: proposals_path,
        active_voteplans,
        challenges,
        committee_keys: Some(committee_keys),
        excluded_proposals,
        output_format: OutputFormat::Csv,
    })?;

    Ok(())
}
