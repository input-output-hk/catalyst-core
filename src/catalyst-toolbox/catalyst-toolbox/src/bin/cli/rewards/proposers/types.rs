use jormungandr_lib::crypto::hash::Hash;
use serde::Serialize;
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

use color_eyre::{eyre::eyre, Report};

macro_rules! bool_enum {
    ($enum_name:ident, $true_case:ident, $false_case:ident) => {
        #[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub enum NotFundedReason {
    #[serde(rename = "Not Funded - Over Budget")]
    OverBudget,
    #[serde(rename = "Not Funded - Approval Threshold")]
    ApprovalThreshold,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct ProposerRewards {
    pub output: PathBuf,
    pub block0: PathBuf,

    pub proposals: Option<PathBuf>,
    pub excluded_proposals: Option<PathBuf>,
    pub active_voteplans: Option<PathBuf>,
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

#[derive(Debug, Serialize)]
pub struct Calculation {
    pub internal_id: String,
    pub proposal_id: Hash,
    pub proposal: String,
    pub overall_score: i64,
    pub yes: u64,
    pub no: u64,
    pub result: u64,
    pub meets_approval_threshold: YesNo,
    pub requested_dollars: i64,
    pub status: FundedStatus,
    pub fund_depletion: f64,
    pub not_funded_reason: Option<NotFundedReason>,
    pub link_to_ideascale: String,
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
