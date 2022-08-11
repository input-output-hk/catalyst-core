use crate::cardano::cli::command::AddressCommand;
use crate::cardano::error::Error;
use crate::config::NetworkType;
use std::io::Write;
use std::path::Path;
use std::process::ExitStatus;
pub struct Address {
    address_command: AddressCommand,
}

impl Address {
    pub fn new(address_command: AddressCommand) -> Self {
        Self { address_command }
    }

    pub fn build<P: AsRef<Path>, Q: AsRef<Path>>(
        self,
        payment_verification_key: P,
        stake_verification_key: P,
        output: Q,
        network: NetworkType,
    ) -> Result<ExitStatus, Error> {
        let output = self
            .address_command
            .build()
            .payment_verification_key_file(payment_verification_key.as_ref())
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
