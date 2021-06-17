mod block0;
mod live;

use block0::Block0StatsCommand;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use jormungandr_testing_utils::testing::block0::GetBlock0Error;
use live::LiveStatsCommand;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub enum IapyxStatsCommand {
    Block0(Block0StatsCommand),
    Live(LiveStatsCommand),
}

impl IapyxStatsCommand {
    pub fn exec(self) -> Result<(), IapyxStatsCommandError> {
        match self {
            Self::Block0(block0) => block0.exec(),
            Self::Live(live) => live.exec(),
        }
    }
}

#[derive(Error, Debug)]
pub enum IapyxStatsCommandError {
    #[error("proxy error")]
    ProxyError(#[from] crate::backend::ProxyServerError),
    #[error("get block0 ")]
    GetBlock0Error(#[from] GetBlock0Error),
    #[error("pin error")]
    PinError(#[from] crate::qr::PinReadError),
    #[error("reqwest error")]
    IapyxStatsCommandError(#[from] reqwest::Error),
    #[error("block0 parse error")]
    Block0ParseError(#[from] Block0ConfigurationError),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("read error")]
    ReadError(#[from] chain_core::mempack::ReadError),
    #[error("bech32 error")]
    Bech32Error(#[from] bech32::Error),
}
