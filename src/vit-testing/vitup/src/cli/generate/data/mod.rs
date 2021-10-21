mod external;
mod perf;
mod random;

use crate::Result;
pub use external::ExternalDataCommandArgs;
pub use jormungandr_lib::interfaces::Initial;
pub use perf::PerfDataCommandArgs;
pub use random::{AllRandomDataCommandArgs, RandomReviewsDataCommandArgs};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum DataCommandArgs {
    /// generate data from external data
    Import(ExternalDataCommandArgs),
    /// generate random data
    Random(RandomDataCommandArgs),
    /// generate data for performance tests
    Perf(PerfDataCommandArgs),
}

impl DataCommandArgs {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Import(import_command) => import_command.exec(),
            Self::Random(random_command) => random_command.exec(),
            Self::Perf(perf_command) => perf_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum RandomDataCommandArgs {
    /// generate all random data
    All(AllRandomDataCommandArgs),
    /// generate reviews random data
    Reviews(RandomReviewsDataCommandArgs),
}

impl RandomDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::All(all_data_command) => all_data_command.exec(),
            Self::Reviews(reviews_random_command) => reviews_random_command.exec(),
        }
    }
}
