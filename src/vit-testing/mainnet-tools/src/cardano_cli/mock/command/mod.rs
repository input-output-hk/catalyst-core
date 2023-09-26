use clap::Parser;
use color_eyre::Report;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod address;
mod query;
mod stake_address;
mod transaction;

pub use address::Address;
pub use query::Query;
pub use stake_address::StakeAddress;
pub use transaction::Transaction;

/// Wrapper around cardano CLI commands
#[derive(Parser, Debug)]
pub enum Command {
    /// Query commands
    #[clap(subcommand)]
    Query(Query),
    /// Address related commands
    #[clap(subcommand)]
    Address(Address),
    /// Stake address related commands
    #[clap(subcommand)]
    StakeAddress(StakeAddress),
    /// Transaction commands
    #[clap(subcommand)]
    Transaction(Transaction),
}

impl Command {
    /// Executes command
    ///
    /// # Errors
    ///
    /// On any sub commands errors
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Self::Query(query) => query.exec().map_err(Into::into),
            Self::Address(address) => address.exec().map_err(Into::into),
            Self::StakeAddress(stake_address) => stake_address.exec().map_err(Into::into),
            Self::Transaction(transaction) => transaction.exec().map_err(Into::into),
        }
    }
}

pub fn write_to_file_or_println(
    maybe_file: Option<PathBuf>,
    content: &str,
) -> Result<(), std::io::Error> {
    if let Some(out_file) = maybe_file {
        let mut file = File::create(out_file)?;
        file.write_all(content.as_bytes())?;
    } else {
        println!("{content}");
    }
    Ok(())
}
