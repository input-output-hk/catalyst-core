use std::str::FromStr;

use color_eyre::{eyre::eyre, Report};

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
