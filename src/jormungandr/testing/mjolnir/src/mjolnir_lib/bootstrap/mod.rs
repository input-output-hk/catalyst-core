mod config;
mod scenario;

use crate::mjolnir_lib::MjolnirError;
use config::{ClientLoadConfig, PassiveBootstrapLoad, ScenarioType};
use jormungandr_automation::jormungandr::grpc::JormungandrClient;
use jormungandr_lib::crypto::hash::Hash;
use std::path::PathBuf;
use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientLoadCommandError {
    #[error("No scenario defined for run. Available: [duration,iteration]")]
    NoScenarioDefined,
    #[error("Client Error")]
    ClientError(#[from] MjolnirError),
}

#[derive(Parser, Debug)]
pub struct ClientLoadCommand {
    /// Number of threads
    #[clap(short = 'c', long = "count", default_value = "3")]
    pub count: u32,
    /// address in format:
    /// /ip4/54.193.75.55/tcp/3000
    #[clap(short = 'a', long = "address")]
    pub address: String,

    /// amount of delay (in seconds) between sync attempts
    #[clap(short = 'p', long = "pace", default_value = "2")]
    pub pace: u64,

    #[clap(short = 'd', long = "storage")]
    pub initial_storage: Option<PathBuf>,

    /// amount of delay (in seconds) between sync attempts
    #[clap(short = 'r', long = "duration")]
    pub duration: Option<u64>,

    /// amount of delay (in seconds) between sync attempts
    #[clap(short = 'n', long = "iterations")]
    pub sync_iteration: Option<u32>,

    #[clap(short = 'm', long = "measure")]
    pub measure: bool,
}

impl ClientLoadCommand {
    pub fn exec(&self) -> Result<(), ClientLoadCommandError> {
        let scenario_type = if let Some(duration) = self.duration {
            Some(ScenarioType::Duration(duration))
        } else {
            self.sync_iteration.map(ScenarioType::Iteration)
        };

        if scenario_type.is_none() {
            return Err(ClientLoadCommandError::NoScenarioDefined);
        }

        let config = self.build_config();

        Ok(PassiveBootstrapLoad::new(config).exec(scenario_type.unwrap())?)
    }

    fn get_block0_hash(&self) -> Hash {
        JormungandrClient::from_address(&self.address)
            .unwrap()
            .get_genesis_block_hash()
            .into()
    }

    fn build_config(&self) -> ClientLoadConfig {
        let block0_hash = self.get_block0_hash();
        ClientLoadConfig::new(
            block0_hash,
            self.measure,
            self.count,
            self.address.clone(),
            self.pace,
            self.initial_storage.clone(),
        )
    }
}
