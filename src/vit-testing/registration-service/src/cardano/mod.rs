use crate::config::Configuration;
use crate::utils::CommandExt;
use jortestkit::prelude::ProcessOutput;
use regex::Regex;
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
        Ok(())
    }

    pub fn tip(&self) -> Result<u64, Error> {
        let regex = r"unSlotNo = (\d+)";

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
        let rg = Regex::new(regex.clone()).map_err(|x| Error::RegexError(x.to_string()))?;
        match rg.captures(&content) {
            Some(x) => Ok(x
                .get(1)
                .ok_or_else(|| {
                    Error::RegexError("wrong regex match no capture of slot-no".to_string())
                })?
                .as_str()
                .parse()
                .map_err(|x: std::num::ParseIntError| Error::ParseIntError(x.to_string()))?),
            None => Err(Error::CannotExtractSlotId {
                regex: regex.to_string(),
                content,
            }),
        }
    }
}

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("cannot extract slot id")]
    CannotExtractSlotId { regex: String, content: String },
    #[error("io error: {0}")]
    CannotCreateAFile(String),
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
