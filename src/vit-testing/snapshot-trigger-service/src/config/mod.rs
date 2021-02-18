use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

const OUTPUT_DIR: &str = "{{RESULT_DIR}}";

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    pub command: Command,
    pub result_dir: PathBuf,
    pub token: Option<String>,
}

impl Configuration {
    pub fn spawn_command(&self, job_id: Uuid) -> Result<std::process::Child, Error> {
        let output_folder = Path::new(&self.result_dir).join(format!("{}", job_id));
        let args: Vec<String> = self
            .command
            .args
            .iter()
            .map(|x| x.replace(OUTPUT_DIR, &format!("{}", output_folder.display())))
            .collect();

        let mut command = std::process::Command::new(&self.command.bin);
        command.args(&args);

        println!("Running command: {:?} ", command);
        command.spawn().map_err(Into::into)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Command {
    pub bin: String,
    pub args: Vec<String>,
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
