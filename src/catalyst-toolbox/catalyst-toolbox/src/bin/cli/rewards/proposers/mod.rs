use std::{collections::HashSet, fs::File};

use catalyst_toolbox::{
    http::HttpClient,
    rewards::proposers::{
        build_path_for_challenge,
        io::{load_data, write_csv, write_json},
        proposer_rewards, OutputFormat, ProposerRewards, ProposerRewardsInputs,
    },
};
use color_eyre::eyre::Result;

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
        vit_station_url,
    }: &ProposerRewards,
    http: &impl HttpClient,
) -> Result<()> {
    let (proposals, voteplans, challenges) = load_data(
        http,
        vit_station_url,
        proposals.as_deref(),
        active_voteplans.as_deref(),
        challenges.as_deref(),
    )?;

    let block0_config = serde_yaml::from_reader(File::open(block0)?)?;

    let excluded_proposals = match excluded_proposals {
        Some(path) => serde_json::from_reader(File::open(path)?)?,
        None => HashSet::new(),
    };
    let committee_keys = match committee_keys {
        Some(path) => serde_json::from_reader(File::open(path)?)?,
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

    for (challenge, calculations) in results {
        let output_path = build_path_for_challenge(output, &challenge.title);

        match output_format {
            OutputFormat::Json => write_json(&output_path, &calculations)?,
            OutputFormat::Csv => write_csv(&output_path, &calculations)?,
        };
    }

    Ok(())
}
