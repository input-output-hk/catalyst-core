use mainnet_tools::snapshot_wormhole::Command;
use structopt::StructOpt;

pub fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    Command::from_args().exec()
}
