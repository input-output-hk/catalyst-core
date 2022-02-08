mod archive;
mod block0;
mod live;
mod snapshot;

use crate::stats::archive::ArchiveCalculatorError;
use crate::stats::archive::ArchiveReaderError;
use archive::ArchiveCommand;
use block0::Block0StatsCommand;
use csv;
use jormungandr_automation::testing::block0::Block0Error;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use live::LiveStatsCommand;
use snapshot::SnapshotCommand;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub enum IapyxStatsCommand {
    Block0(Block0StatsCommand),
    Live(LiveStatsCommand),
    Archive(ArchiveCommand),
    Snapshot(SnapshotCommand),
}

impl IapyxStatsCommand {
    pub fn exec(self) -> Result<(), IapyxStatsCommandError> {
        match self {
            Self::Block0(block0) => block0.exec(),
            Self::Live(live) => live.exec(),
            Self::Archive(archive) => archive.exec(),
            Self::Snapshot(snapshot) => snapshot.exec(),
        }
    }
}

#[derive(Error, Debug)]
pub enum IapyxStatsCommandError {
    #[error("get block0")]
    GetBlock0(#[from] Block0Error),
    #[error("pin error")]
    Pin(#[from] crate::qr::PinReadError),
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
}
