pub mod archive;
pub mod distribution;
pub mod live;
pub mod snapshot;
pub mod voters;

use crate::stats::archive::{ArchiveCalculatorError, ArchiveReaderError};
use jormungandr_automation::testing::block0::Block0Error;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("get block0")]
    GetBlock0(#[from] Block0Error),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("block0 parse error")]
    Block0Parse(#[from] Block0ConfigurationError),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("read error")]
    Read(#[from] chain_core::mempack::ReadError),
    #[error("bech32 error")]
    Bech32(#[from] bech32::Error),
    #[error("csv error")]
    Csv(#[from] csv::Error),
    #[error("archive reader error")]
    ArchiveReader(#[from] ArchiveReaderError),
    #[error("archive calculator error")]
    ArchiveCalculator(#[from] ArchiveCalculatorError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Live(#[from] live::Error),
}
