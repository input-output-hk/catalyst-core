use catalyst_toolbox::{
    rewards::proposers::{
        io::write_results, proposer_rewards, ProposerRewards, ProposerRewardsInputs,
    },
    utils::json_from_file,
};
use color_eyre::eyre::Result;
use std::{collections::HashSet, fs::File};

pub fn rewards(
    ProposerRewards {
        output,
        block0,
        proposals,
        excluded_proposals,
        active_voteplans,
        challenges,
        committee_keys,
        total_stake_threshold,
        approval_threshold,
        output_format,
    }: &ProposerRewards,
) -> Result<()> {
    let proposals = json_from_file(proposals)?;
    let voteplans = json_from_file(active_voteplans)?;
    let challenges = json_from_file(challenges)?;

    let block0_config = serde_yaml::from_reader(File::open(block0)?)?;

    let excluded_proposals = match excluded_proposals {
        Some(path) => json_from_file(path)?,
        None => HashSet::new(),
    };
    let committee_keys = match committee_keys {
        Some(path) => json_from_file(path)?,
        None => vec![],
    };

    let results = proposer_rewards(ProposerRewardsInputs {
        block0_config,
        proposals,
        voteplans,
        challenges,
        excluded_proposals,
        committee_keys,
        total_stake_threshold: *total_stake_threshold,
        approval_threshold: *approval_threshold,
    })?;

    write_results(output, *output_format, results)?;

    Ok(())
}
