mod job;

pub use job::JobParameters;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    #[serde(rename = "voting-tools")]
    pub voting_tools: VotingToolsParams,
    #[serde(rename = "result-dir")]
    pub result_dir: PathBuf,
    pub token: Option<String>,
}

impl Configuration {
    pub fn spawn_command(
        &self,
        job_id: Uuid,
        params: JobParameters,
    ) -> Result<std::process::Child, Error> {
        let output_folder = Path::new(&self.result_dir).join(format!("{}", job_id));
        let mut command = std::process::Command::new(&self.voting_tools.bin);
        command.arg("genesis");
        match self.voting_tools.network {
            NetworkType::Mainnet => command.arg("--mainnet"),
            NetworkType::Testnet(magic) => command.arg("--testnet").arg(magic.to_string()),
        };

        command
            .arg("--db")
            .arg(&self.voting_tools.db)
            .arg("--db-user")
            .arg(&self.voting_tools.db_user)
            .arg("--db-host")
            .arg(&self.voting_tools.db_host)
            .arg("--out-file")
            .arg(output_folder.join("snapshot.json"))
            .arg("--scale")
            .arg(self.voting_tools.scale.to_string())
            .arg("--slot-id")
            .arg(params.slot_id.to_string())
            .arg("--threshold")
            .arg(params.threshold.to_string());

        println!("Running command: {:?} ", command);
        command.spawn().map_err(Into::into)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct VotingToolsParams {
    pub bin: String,
    pub network: NetworkType,
    pub db: String,
    #[serde(rename = "db-user")]
    pub db_user: String,
    #[serde(rename = "db-host")]
    pub db_host: String,
    pub scale: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum NetworkType {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "testnet")]
    Testnet(u32),
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Configuration, Error> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse configuration")]
    CannotParseConfiguration(#[from] serde_json::Error),
    #[error("cannot read configuration: {0:?}")]
    CannotReadConfiguration(PathBuf),
    #[error("cannot spawn command")]
    CannotSpawnCommand(#[from] std::io::Error),
}
