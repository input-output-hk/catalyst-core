use clap::Parser;
use jormungandr_lib::crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

use color_eyre::{eyre::eyre, Report};

macro_rules! bool_enum {
    ($enum_name:ident, $true_case:ident, $false_case:ident) => {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum $enum_name {
            $true_case,
            $false_case,
        }

        impl From<bool> for $enum_name {
            fn from(b: bool) -> Self {
                match b {
                    true => $enum_name::$true_case,
                    false => $enum_name::$false_case,
                }
            }
        }
    };
}

bool_enum!(YesNo, Yes, No);
bool_enum!(FundedStatus, Funded, NotFunded);

#[derive(Debug, Serialize, Deserialize)]
pub enum NotFundedReason {
    #[serde(rename = "Not Funded - Over Budget")]
    OverBudget,
    #[serde(rename = "Not Funded - Approval Threshold")]
    ApprovalThreshold,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct ProposerRewards {
    #[clap(long = "output-file")]
    pub output: PathBuf,

    #[clap(long = "block0-path")]
    pub block0: PathBuf,

    #[clap(default_value = "0.01")]
    #[clap(long)]
    pub total_stake_threshold: f64,

    #[clap(default_value = "1.15")]
    #[clap(long)]
    pub approval_threshold: f64,

    #[clap(default_value = "csv")]
    #[clap(long)]
    pub output_format: OutputFormat,

    #[clap(long = "proposals-path")]
    pub proposals: PathBuf,
    #[clap(long = "excluded-proposals-path")]
    pub excluded_proposals: Option<PathBuf>,
    #[clap(long = "active-voteplan-path")]
    pub active_voteplans: PathBuf,
    #[clap(long = "challenges-path")]
    pub challenges: PathBuf,

    #[clap(long = "committee-keys-path")]
    pub committee_keys: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Calculation {
    pub internal_id: String,
    pub proposal_id: Hash,
    pub proposal: String,
    pub overall_score: i64,
    pub yes: u64,
    pub no: u64,
    pub result: i64,
    pub meets_approval_threshold: YesNo,
    pub requested_dollars: i64,
    pub status: FundedStatus,
    pub fund_depletion: f64,
    pub not_funded_reason: Option<NotFundedReason>,
    pub link_to_ideascale: String,
}

#[cfg(test)]
impl Default for Calculation {
    fn default() -> Self {
        Self {
            internal_id: Default::default(),
            proposal_id: <[u8; 32]>::default().into(),
            proposal: Default::default(),
            overall_score: Default::default(),
            yes: Default::default(),
            no: Default::default(),
            result: Default::default(),
            meets_approval_threshold: YesNo::Yes,
            requested_dollars: Default::default(),
            status: FundedStatus::NotFunded,
            fund_depletion: Default::default(),
            not_funded_reason: None,
            link_to_ideascale: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Csv,
}

impl FromStr for OutputFormat {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            s => Err(eyre!("expected one of `csv` or `json`, found {s}")),
        }
    }
}
