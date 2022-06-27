use std::path::PathBuf;

use color_eyre::Result;
use log::info;
use structopt::StructOpt;

use self::types::OutputFormat;

mod types;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ProposalInputs {}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ProposerRewards {
    pub output: PathBuf,
    pub block0: PathBuf,

    pub proposals: Option<PathBuf>,
    pub excluded_proposasl: Option<PathBuf>,
    pub active_voteplan: Option<PathBuf>,
    pub challenges: Option<PathBuf>,
    pub committee_keys: Option<PathBuf>,

    #[structopt(default_value = "0.01")]
    pub total_stake_threshold: f64,

    #[structopt(default_value = "1.15")]
    pub approval_threshold: f64,

    pub output_format: OutputFormat,

    #[structopt(default_value = "https://servicing-station.vit.iohk.io")]
    pub vit_station_url: String,
}

pub fn rewards(
    ProposerRewards {
        output,
        block0,
        proposals,
        excluded_proposasl,
        active_voteplan,
        challenges,
        committee_keys,
        total_stake_threshold,
        approval_threshold,
        output_format,
        vit_station_url,
    }: &ProposerRewards,
) -> Result<()> {
    match inputs {
        Some(inputs) => info!("using local files"),
        None => info!("using data from {vit_station_url}"),
    }

    Ok(())
}
