use clap::Parser;
use mainnet_tools::snapshot::wormhole::Command;

pub fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    Command::parse().exec()
}
