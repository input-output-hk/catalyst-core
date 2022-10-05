use crate::Error;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone)]
pub struct CatalystToolboxCli {
    path: PathBuf,
}

impl CatalystToolboxCli {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn generate_qr<P: AsRef<Path>>(
        &self,
        secret_key_path: P,
        pin: String,
        out_file: P,
    ) -> Result<ExitStatus, Error> {
        let mut command = Command::new(&self.path);
        command
            .arg("qr-code")
            .arg("encode")
            .arg("--input")
            .arg(secret_key_path.as_ref())
            .arg("--output")
            .arg(out_file.as_ref())
            .arg("--pin")
            .arg(pin)
            .arg("img");

        println!("Running catalyst-toolbox: {:?}", command);

        Ok(command.output()?.status)
    }
}
