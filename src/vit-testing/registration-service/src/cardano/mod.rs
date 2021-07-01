use crate::config::Configuration;
use crate::utils::CommandExt;
use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use thiserror::Error;
use uuid::Uuid;

pub struct CardanoCliExecutor {
    config: Configuration,
}

impl CardanoCliExecutor {
    pub fn new(config: Configuration) -> Self {
        Self { config }
    }

    pub fn transaction_submit(&self, raw: Vec<u8>) -> Result<(), Error> {
        let id = Uuid::new_v4();
        let mut tx_signed_file = self.config.result_dir.clone();
        tx_signed_file.push(id.to_string());
        tx_signed_file.push("tx.signed");

        std::fs::create_dir_all(tx_signed_file.parent().unwrap())
            .map_err(|x| Error::CannotCreateParentDirectory(x.to_string()))?;

        let mut file =
            File::create(&tx_signed_file).map_err(|x| Error::CannotCreateAFile(x.to_string()))?;
        file.write_all(&raw)
            .map_err(|x| Error::CannotWriteAFile(x.to_string()))?;

        let mut command = Command::new(&self.config.cardano_cli);
        command
            .arg("transaction")
            .arg("submit")
            .arg("--tx-file")
            .arg(&tx_signed_file)
            .arg_network(self.config.network);

        println!("Running cardano_cli: {:?}", command);

        let content = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?
            .as_lossy_string();

        println!("Output of cardano_cli: {:?}", content);

        Ok(())
    }

    pub fn tip(&self) -> Result<String, Error> {
        let mut command = Command::new(&self.config.cardano_cli);
        command
            .arg("query")
            .arg("tip")
            .arg_network(self.config.network);

        println!("querying tip: {:?}", command);

        let content = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?
            .as_lossy_string();
        println!("raw output of query tip: {}", content);
        Ok(content)
    }

    pub fn query_utxo<S: Into<String>>(&self, payment_address: S) -> Result<String, Error> {
        let mut command = Command::new(&self.config.cardano_cli);
        command
            .arg("query")
            .arg("utxo")
            .arg_network(self.config.network)
            .arg("--address")
            .arg(payment_address.into())
            .arg("--out-file")
            .arg("/dev/stdout");

        println!("Running cardano_cli: {:?}", command);

        let content = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?
            .as_lossy_string();

        println!("raw output of query utxo: {}", content);
        Ok(content)
    }
}

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("cannot extract slot id")]
    CannotExtractSlotId { regex: String, content: String },
    #[error("io error: {0}")]
    CannotCreateAFile(String),
    #[error("io error: {0}")]
    CannotCreateParentDirectory(String),
    #[error("io error: {0}")]
    CannotWriteAFile(String),
    #[error("io error: {0}")]
    CannotGetOutputFromCommand(String),
    #[error("parse error: {0}")]
    ParseIntError(String),
    #[error("cannot parse voter-registration output: {0:?}")]
    RegexError(String),
    #[error("cannot parse cardano cli output: {0:?}")]
    CannotParseCardanoCliOutput(String),
}
