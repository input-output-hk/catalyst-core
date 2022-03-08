use pyo3::prelude::*;
use std::{path::PathBuf, str::FromStr};

#[derive(Debug)]
pub struct ProposerRewardsExecutor {
    output_file: PathBuf,
    block0_path: PathBuf,
    total_stake_threshold: f64,
    approval_threshold: f64,
    output_format: String,
    committee_keys_path: Option<String>,
    proposals_path: Option<String>,
    excluded_proposals_path: Option<String>,
    active_voteplan_path: Option<String>,
    challenges_path: Option<String>,
    vit_station_url: String,
}

impl Default for ProposerRewardsExecutor {
    fn default() -> Self {
        Self {
            output_file: PathBuf::from_str("./output").unwrap(),
            block0_path: PathBuf::from_str("./block0.bin").unwrap(),
            total_stake_threshold: 0.01,
            approval_threshold: 1.15,
            output_format: "csv".to_string(),
            committee_keys_path: None,
            proposals_path: None,
            excluded_proposals_path: None,
            active_voteplan_path: None,
            challenges_path: None,
            vit_station_url: "https://servicing-station.vit.iohk.io".to_string(),
        }
    }
}

impl ProposerRewardsExecutor {
    pub fn output_file(mut self, output_file: PathBuf) -> Self {
        self.output_file = output_file;
        self
    }
    pub fn block0_path(mut self, block0_path: PathBuf) -> Self {
        self.block0_path = block0_path;
        self
    }
    pub fn total_stake_threshold(mut self, total_stake_threshold: f64) -> Self {
        self.total_stake_threshold = total_stake_threshold;
        self
    }
    pub fn approval_threshold(mut self, approval_threshold: f64) -> Self {
        self.approval_threshold = approval_threshold;
        self
    }
    pub fn output_format(mut self, output_format: String) -> Self {
        self.output_format = output_format;
        self
    }
    pub fn proposals_path(mut self, proposals_path: String) -> Self {
        self.proposals_path = Some(proposals_path);
        self
    }
    pub fn excluded_proposals_path(mut self, excluded_proposals_path: String) -> Self {
        self.excluded_proposals_path = Some(excluded_proposals_path);
        self
    }

    pub fn committee_keys_path(mut self, committee_keys_path: String) -> Self {
        self.committee_keys_path = Some(committee_keys_path);
        self
    }

    pub fn active_voteplan_path(mut self, active_voteplan_path: String) -> Self {
        self.active_voteplan_path = Some(active_voteplan_path);
        self
    }
    pub fn challenges_path(mut self, challenges_path: String) -> Self {
        self.challenges_path = Some(challenges_path);
        self
    }
    pub fn vit_station_url(mut self, vit_station_url: String) -> Self {
        self.vit_station_url = vit_station_url;
        self
    }

    pub fn proposers_rewards(self) -> PyResult<()> {
        Python::with_gil(|py| {
            let fun: Py<PyAny> = PyModule::from_code(
                py,
                include_str!("../scripts/python/proposers_rewards.py"),
                "",
                "",
            )?
            .getattr("calculate_rewards")?
            .into();

            let args = (
                self.output_file.to_str(),
                self.block0_path.to_str(),
                self.total_stake_threshold.to_string(),
                self.approval_threshold.to_string(),
                self.output_format.to_string(),
                self.proposals_path,
                self.excluded_proposals_path,
                self.active_voteplan_path,
                self.challenges_path,
                self.vit_station_url,
                self.committee_keys_path,
            );
            fun.call1(py, args)
        })?;
        Ok(())
    }
}
