use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

mod address;
mod query;
mod stake_address;
mod transaction;

pub use address::AddressCommand;
pub use query::QueryCommand;
pub use stake_address::StakeAddressCommand;
pub use transaction::TransactionCommand;

#[derive(StructOpt, Debug)]
pub enum CardanoCliCommand {
    Query(QueryCommand),
    Address(AddressCommand),
    StakeAddress(StakeAddressCommand),
    Transaction(TransactionCommand),
}

impl CardanoCliCommand {
    pub fn exec(self) -> Result<(), Error> {
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
    content: String,
) -> Result<(), std::io::Error> {
    if let Some(out_file) = maybe_file {
        let mut file = File::create(out_file)?;
        file.write_all(content.as_bytes())?;
    } else {
        println!("{}", content);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parsing(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
