use crate::cardano_cli::wrapper::cli::command;
use crate::cardano_cli::wrapper::Error;
use jortestkit::prelude::ProcessOutput;
use snapshot_trigger_service::config::NetworkType;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use tracing::debug;
use uuid::Uuid;

pub struct Transaction {
    transaction_command: command::Transaction,
}

impl Transaction {
    pub fn new(transaction_command: command::Transaction) -> Self {
        Self {
            transaction_command,
        }
    }

    pub fn sign<P: AsRef<Path>, Q: AsRef<Path>>(
        self,
        tx_raw: P,
        payment_skey_file: P,
        output_file: Q,
    ) -> Result<ExitStatus, Error> {
        let mut command = self
            .transaction_command
            .sign()
            .tx_body_file(tx_raw)
            .signing_key_file(payment_skey_file)
            .out_file(output_file)
            .build();
        let output = command.output().map_err(|e| Error::Io(e.to_string()))?;

        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(output.status)
    }

    pub fn submit_from_bytes<P: AsRef<Path>>(
        self,
        raw: &[u8],
        working_dir: P,
    ) -> Result<String, Error> {
        let id = Uuid::new_v4();
        let mut tx_signed_file = working_dir.as_ref().to_path_buf();
        tx_signed_file.push(id.to_string());
        tx_signed_file.push("tx.signed");

        std::fs::create_dir_all(
            tx_signed_file
                .parent()
                .ok_or_else(|| Error::CannotGetParentDirectory(tx_signed_file.clone()))?,
        )
        .map_err(|x| Error::CannotCreateParentDirectory(x.to_string()))?;

        let mut file =
            File::create(&tx_signed_file).map_err(|x| Error::CannotCreateAFile(x.to_string()))?;
        file.write_all(raw)
            .map_err(|x| Error::CannotWriteAFile(x.to_string()))?;

        self.submit(&tx_signed_file)
    }

    pub fn submit<P: AsRef<Path>>(self, tx_signed: P) -> Result<String, Error> {
        let mut command = self.transaction_command.submit().tx_file(tx_signed).build();
        let output = command
            .output()
            .map_err(|_| Error::CannotGetOutputFromCommand(format!("{command:?}")))?;

        debug!("status: {}", output.status);
        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(output.as_lossy_string())
    }

    pub fn id<P: AsRef<Path>>(self, tx_signed: P) -> Result<String, Error> {
        let mut command = self.transaction_command.id().tx_file(tx_signed).build();
        let output = command
            .output()
            .map_err(|_| Error::CannotGetOutputFromCommand(format!("{command:?}")))?;

        println!("status: {}", output.status);
        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(output.as_lossy_string())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        self,
        network: NetworkType,
        tx_in: String,
        change_address: String,
        certificate_file: PathBuf,
        protocol_params_file: PathBuf,
        out_file: PathBuf,
        witness_override: u32,
    ) -> Result<ExitStatus, Error> {
        let mut command = self
            .transaction_command
            .build()
            .network(network)
            .tx_in(tx_in)
            .change_address(change_address)
            .certificate_file(certificate_file)
            .protocol_params_file(protocol_params_file)
            .out_file(out_file)
            .witness_override(witness_override)
            .build();

        let output = command
            .output()
            .map_err(|_| Error::CannotGetOutputFromCommand(format!("{command:?}")))?;

        std::io::stdout()
            .write_all(&output.stdout)
            .map_err(|e| Error::Io(e.to_string()))?;
        std::io::stderr()
            .write_all(&output.stderr)
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(output.status)
    }
}
