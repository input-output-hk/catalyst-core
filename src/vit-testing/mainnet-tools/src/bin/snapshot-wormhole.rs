use mainnet_tools::snapshot_wormhole::Command;
use structopt::StructOpt;

pub fn main() -> Result<(), color_eyre::Report> {
    Command::from_args().exec()
}
