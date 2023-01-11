use mainnet_tools::snapshot::wormhole::Command;
use clap::Parser;

pub fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    Command::parse().exec()
}
