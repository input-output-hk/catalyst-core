use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::ExitStatus;
use snapshot_trigger_service::config::NetworkType;
use crate::cardano_cli::mock::fake::Tip;
use crate::cardano_cli::wrapper::cli::command;
use crate::cardano_cli::wrapper::Error;

pub struct Query {
    query_command: command::Query,
}

impl Query {
    pub fn new(query_command: command::Query) -> Self {
        Self { query_command }
    }

    pub fn tip(self, network: NetworkType) -> Result<Tip, Error> {
        let mut command = self.query_command.tip(network).build();
        let output = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?;

        println!("raw output of query tip: {}", output.as_lossy_string());
        serde_json::from_str(&output.as_lossy_string()).map_err(|e| Error::Serde(e.to_string()))
    }

    pub fn utxo<S: Into<String>>(
        self,
        payment_address: S,
        network: NetworkType,
    ) -> Result<Utxo, Error> {
        let mut command = self.query_command.utxo(network, payment_address).build();
        let output = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?;

        println!("raw output of query utxo: {}", output.as_lossy_string());
        Ok(Utxo::from_string(&output.as_multi_line()))
    }

    pub fn protocol_parameters<P: AsRef<Path>>(
        self,
        network: NetworkType,
        out_file: P,
    ) -> Result<ExitStatus, Error> {
        let mut command = self
            .query_command
            .protocol_parameters(network, out_file)
            .build();
        let output = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?;

        println!(
            "raw output of query protocol parameters: {}",
            output.as_lossy_string()
        );
        Ok(output.status)
    }

    pub fn funds<S: Into<String>>(
        self,
        payment_address: S,
        network: NetworkType,
    ) -> Result<u64, Error> {
        Ok(self.utxo(payment_address, network)?.get_total_funds())
    }
}

pub struct Utxo {
    pub entries: Vec<UtxoEntry>,
}

/// Supported output
/// ------------------------------------------------------------------ ------ -------------------
///  `TxHash`                                                           | `TxIx` | Amount
///  61d47e568b1502064906e977aae848c7aec9a76f97e7d11ad5d752e95c438011 | 0    | 1379280 lovelace
///  ac1d8802a4e100d90ce59fb4e4573f1c7884a65197ff39810a88eb0b07de3aa6 | 0    | 30000000 lovelace
///  69818d49963ffafe8a287ec270d05ba89493de33ddf7b5b9bcb07e97802a0f28 | 0    | 5573009 lovelace
///  fba1526c49684722199b102bffd5b4a66ea1d490605532753fa24e12af925722 | 0    | 5000000 lovelace
/// ------------------------------------------------------------------ ------ -------------------
impl Utxo {
    fn from_string(output: &[String]) -> Self {
        Self {
            entries: output
                .iter()
                .filter_map(|row| {
                    if row.contains("---") || row.contains("TxHash") {
                        None
                    } else {
                        let items: Vec<&str> = row.split_whitespace().collect();

                        Some(UtxoEntry {
                            hash: items[0].to_string(),
                            index: items[2].parse().unwrap(),
                            amount: items[4].parse().unwrap(),
                        })
                    }
                })
                .collect(),
        }
    }

    pub fn get_total_funds(&self) -> u64 {
        self.entries.iter().map(|entry| entry.amount).sum()
    }
}

pub struct UtxoEntry {
    pub hash: String,
    pub index: u32,
    pub amount: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsResponse {
    #[serde(flatten)]
    content: HashMap<String, FundsEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsEntry {
    address: String,
    value: FundsValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsValue {
    lovelace: u64,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_funds_extraction() {
        let content =  vec!["------------------------------------------------------------------ ------ -------------------".to_string(),
        "    TxHash                                                           | TxIx | Amount".to_string(),
        "61d47e568b1502064906e977aae848c7aec9a76f97e7d11ad5d752e95c438011 | 0    | 1379280 lovelace".to_string(),
        "ac1d8802a4e100d90ce59fb4e4573f1c7884a65197ff39810a88eb0b07de3aa6 | 0    | 30000000 lovelace".to_string(),
        "69818d49963ffafe8a287ec270d05ba89493de33ddf7b5b9bcb07e97802a0f28 | 0    | 5573009 lovelace".to_string(),
        "fba1526c49684722199b102bffd5b4a66ea1d490605532753fa24e12af925722 | 0    | 5000000 lovelace".to_string(),
        "    ------------------------------------------------------------------ ------ -------------------".to_string()];
        assert_eq!(Utxo::from_string(&content).get_total_funds(), 41_952_289);
    }
}
