mod job;

use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
pub use job::JobParameters;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    #[serde(flatten)]
    pub inner: scheduler_service_lib::Configuration,
    #[serde(rename = "voting-tools")]
    pub voting_tools: VotingToolsParams,
}

impl Configuration {
    pub fn set_token(&mut self, token: Option<String>) {
        self.inner.api_token = token;
    }

        let mut command = self.voting_tools.command()?;
        match self.voting_tools.network {
            NetworkType::Mainnet => command.arg("--mainnet"),
            NetworkType::Testnet(magic) => command.arg("--testnet-magic").arg(magic.to_string()),
        };

        let output_filename = self.crate_snapshot_output_file_name(&params.tag);

        command
            .arg("--db")
            .arg(&self.voting_tools.db)
            .arg("--db-user")
            .arg(&self.voting_tools.db_user)
            .arg("--db-pass")
            .arg(&self.voting_tools.db_pass)
            .arg("--db-host")
            .arg(&self.voting_tools.db_host)
            .arg("--out-file")
            .arg(output_folder.join(output_filename))
            .arg("--scale")
            .arg(self.voting_tools.scale.to_string());

        if let Some(slot_no) = params.slot_no {
            command.arg("--slot-no").arg(slot_no.to_string());
        }

        if !self.voting_tools.additional_params.is_empty() {
            command.args(&self.voting_tools.additional_params);
        }

        self.print_with_password_hidden(&command);

        command.spawn().map_err(Into::into)
    }

    pub fn result_dir(&self) -> PathBuf {
        self.inner.result_dir.clone()

    pub fn address_mut(&mut self) -> &mut SocketAddr {
        &mut self.inner.address
    }

    pub fn address(&self) -> &SocketAddr {
        &self.inner.address
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    pub fn with_port(mut self, port: u16) -> Self {
        self.configuration.inner.address.set_port(port);
        self
    }

    pub fn with_result_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.configuration.inner.result_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn with_tmp_result_dir(self, tmp: &TempDir) -> Self {
        self.with_result_dir(tmp.child("snapshot_result").path())
    }

    pub fn with_voting_tools_params(mut self, voting_tools: VotingToolsParams) -> Self {
        self.configuration.voting_tools = voting_tools;
        self
    }

    pub fn build(self) -> Configuration {
        self.configuration
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            inner: scheduler_service_lib::Configuration {
                result_dir: Path::new(".").to_path_buf(),
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7070),
                api_token: None,
                admin_token: None,
                working_directory: None,
            },
            voting_tools: VotingToolsParams {
                bin: None,
                nix_branch: None,
                network: NetworkType::Mainnet,
                db: "".to_string(),
                db_user: "".to_string(),
                db_pass: "".to_string(),
                db_host: "".to_string(),
                scale: 1_000_000,
                additional_params: vec![],
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct VotingToolsParams {
    /// binary name
    pub bin: Option<String>,
    /// in some occasion we need to run voting-tools via some dependency management
    #[serde(rename = "nix-branch")]
    pub nix_branch: Option<String>,
    /// network type
    pub network: NetworkType,
    /// db name
    pub db: String,
    #[serde(rename = "db-user")]
    /// db user
    pub db_user: String,
    #[serde(rename = "db-pass")]
    /// db pass
    pub db_pass: String,
    /// db host
    #[serde(rename = "db-host")]
    pub db_host: String,
    /// voting power scale. If 1 then voting power will be expressed in Lovelace
    pub scale: u32,
    /// additional parameters
    pub additional_params: Vec<String>,
}

impl VotingToolsParams {
    pub fn command(&self) -> Result<std::process::Command, Error> {
        if let Some(bin) = &self.bin {
            return Ok(std::process::Command::new(bin));
        } else if let Some(nix_branch) = &self.nix_branch {
            let mut command = std::process::Command::new("nix");
            command.arg("run");
            command.arg(nix_branch);
            command.arg("--");
            return Ok(command);
        }
        Err(Error::WrongVotingToolsConfiguration)
    }
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

pub fn write_config<P: AsRef<Path>>(config: Configuration, path: P) -> Result<(), Error> {
    use std::io::Write;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())
        .map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse configuration")]
    CannotParseConfiguration(#[from] serde_json::Error),
    #[error("cannot read configuration: {0:?}")]
    CannotReadConfiguration(PathBuf),
    #[error("cannot spawn command")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot find voting tools at {0:?}")]
    CannotFindVotingTools(PathBuf),
    #[error("no 'bin' or 'run-through' defined in voting tools")]
    WrongVotingToolsConfiguration,
    #[error("result folder does not exists at {0:?}")]
    ResultFolderDoesNotExists(PathBuf),
}
