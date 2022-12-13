use crate::cardano_cli::wrapper::cli::command;
use crate::cardano_cli::wrapper::Error;
use snapshot_trigger_service::config::NetworkType;
use std::io::Write;
use std::path::Path;
use std::process::ExitStatus;

pub struct StakeAddress {
    stake_address_command: command::StakeAddress,
}

impl StakeAddress {
    pub fn new(stake_address_command: command::StakeAddress) -> Self {
        Self {
            stake_address_command,
        }
    }

    pub fn register_certificate<P: AsRef<Path>, Q: AsRef<Path>>(
        self,
        stake_verification_key_file: P,
        output: Q,
    ) -> Result<ExitStatus, Error> {
        let output = self
            .stake_address_command
            .register_certificate()
            .stake_verification_key_file(stake_verification_key_file.as_ref())
            .out_file(output.as_ref())
            .build()
            .output()
            .map_err(|e| Error::Io(e.to_string()))?;

        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(output.status)
    }

    pub fn build<P: AsRef<Path>, Q: AsRef<Path>>(
        self,
        stake_verification_key: P,
        output: Q,
        network: NetworkType,
    ) -> Result<ExitStatus, Error> {
        let output = self
            .stake_address_command
            .build()
            .stake_verification_key_file(stake_verification_key.as_ref())
            .out_file(output.as_ref())
            .network(network)
            .build()
            .output()
            .map_err(|e| Error::Io(e.to_string()))?;

        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(output.status)
    }
}
