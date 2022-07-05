use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use catalyst_toolbox::{
    http::HttpClient,
    rewards::proposers::{
        proposer_rewards, Calculation, OutputFormat, ProposerRewards, ProposerRewardsInputs, build_path_for_challenge,
    },
};
use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use self::util::load_data;

mod util;

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

pub fn write_json(path: &Path, results: &[Calculation]) -> Result<()> {
    let writer = BufWriter::new(File::options().write(true).open(path)?);
    serde_json::to_writer(writer, &results)?;

    Ok(())
}

pub fn write_csv(path: &Path, results: &[Calculation]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in results {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}
