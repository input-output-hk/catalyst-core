use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use std::process::Command;
use std::{path::PathBuf, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
pub struct ProposerRewardsCommand {
    python_exec: String,
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
    vit_station_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Default for ProposerRewardsCommand {
    fn default() -> Self {
        Self {
            python_exec: "python3".to_string(),
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
            vit_station_url: None,
        }
    }
}

impl ProposerRewardsCommand {
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
        self.vit_station_url = Some(vit_station_url);
        self
    }

    pub fn cmd(self, temp_dir: &TempDir) -> Result<Command, Error> {
        let script_content = include_str!("../scripts/python/proposers_rewards.py");
        let script_file = temp_dir.child("proposers_rewards.py");

        std::fs::write(script_file.path(), script_content)?;

        let mut command = Command::new(self.python_exec);

        command
            .arg(script_file.path())
            .arg("--output-file")
            .arg(self.output_file)
            .arg("--block0-path")
            .arg(self.block0_path)
            .arg("--output-format")
            .arg(self.output_format)
            .arg("--total-stake-threshold")
            .arg(self.total_stake_threshold.to_string())
            .arg("--approval-threshold")
            .arg(self.approval_threshold.to_string());

        if let Some(proposals_path) = self.proposals_path {
            command.arg("--proposals-path").arg(proposals_path);
        }

        if let Some(active_voteplan_path) = self.active_voteplan_path {
            command
                .arg("--active-voteplan-path")
                .arg(active_voteplan_path);
        }

        if let Some(challenges_path) = self.challenges_path {
            command.arg("--challenges-path").arg(challenges_path);
        }

        if let Some(excluded_proposals_path) = self.excluded_proposals_path {
            command
                .arg("--excluded-proposals-path")
                .arg(excluded_proposals_path);
        }
        if let Some(committee_keys_path) = self.committee_keys_path {
            command
                .arg("--committee-keys-path")
                .arg(committee_keys_path);
        }
        if let Some(vit_station_url) = self.vit_station_url {
            command.arg("--vit-station-url").arg(vit_station_url);
        }

        Ok(command)
    }
}
