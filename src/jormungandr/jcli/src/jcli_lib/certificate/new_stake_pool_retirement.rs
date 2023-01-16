use crate::jcli_lib::certificate::{write_cert, Error};
use chain_crypto::Blake2b256;
use chain_impl_mockchain::certificate::{Certificate, PoolRetirement};
use chain_time::DurationSeconds;
use clap::Parser;
use std::path::PathBuf;

/// retire the given stake pool ID From the blockchain
///
/// by doing so all remaining stake delegated to this stake pool will
/// become pending and will need to be re-delegated.
#[derive(Parser)]
pub struct StakePoolRetirement {
    /// set the 32bytes (in hexadecimal) of the Stake Pool identifier
    #[clap(long = "pool-id", name = "POOL_ID")]
    pool_id: Blake2b256,

    /// start retirement
    ///
    /// This state when the stake pool retirement becomes effective in seconds since
    /// the block0 start time.
    #[clap(long = "retirement-time", name = "SECONDS-SINCE-START")]
    pub retirement_time: u64,

    /// print the output signed certificate in the given file, if no file given
    /// the output will be printed in the standard output
    pub output: Option<PathBuf>,
}

impl StakePoolRetirement {
    pub fn exec(self) -> Result<(), Error> {
        let pool_retirement = PoolRetirement {
            pool_id: self.pool_id.into(),
            retirement_time: DurationSeconds::from(self.retirement_time).into(),
        };

        let cert = Certificate::PoolRetirement(pool_retirement);
        write_cert(self.output, cert.into())
    }
}
