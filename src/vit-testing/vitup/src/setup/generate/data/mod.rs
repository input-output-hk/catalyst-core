mod external;
mod perf;
mod random;

pub use external::ExternalDataCommandArgs;
pub use jormungandr_lib::interfaces::Initial;
pub use perf::PerfDataCommandArgs;
pub use random::RandomDataCommandArgs;

use crate::config::DataGenerationConfig;
use crate::Result;
use chain_core::property::Block;
use chain_core::property::Serialize;
use chain_impl_mockchain::ledger::Ledger;
use jormungandr_lib::interfaces::Block0Configuration;
use std::path::Path;

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

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<DataGenerationConfig> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn read_genesis_yaml<P: AsRef<Path>>(genesis: P) -> Result<Block0Configuration> {
    let contents = std::fs::read_to_string(&genesis)?;
    serde_yaml::from_str(&contents).map_err(Into::into)
}

pub fn read_initials<P: AsRef<Path>>(initials: P) -> Result<Vec<Initial>> {
    let contents = std::fs::read_to_string(&initials)?;
    let value: serde_json::Value = serde_json::from_str(&contents)?;
    let initial = serde_json::to_string(&value["initial"])?;
    serde_json::from_str(&initial).map_err(Into::into)
}

pub fn write_genesis_yaml<P: AsRef<Path>>(genesis: Block0Configuration, path: P) -> Result<()> {
    use std::io::Write;
    let content = serde_yaml::to_string(&genesis)?;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn encode<P: AsRef<Path>, Q: AsRef<Path>>(genesis: P, block0: Q) -> Result<()> {
    let input: std::fs::File = std::fs::OpenOptions::new()
        .create(false)
        .write(false)
        .read(true)
        .append(false)
        .truncate(false)
        .open(&genesis)?;

    let output: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .append(false)
        .truncate(true)
        .open(&block0)?;

    let genesis: Block0Configuration = serde_yaml::from_reader(input)?;
    let block = genesis.to_block();
    Ledger::new(block.id(), block.fragments())?;
    block.serialize(&output).map_err(Into::into)
}
