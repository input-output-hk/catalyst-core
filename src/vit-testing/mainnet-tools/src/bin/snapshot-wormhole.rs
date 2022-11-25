use mainnet_tools::snapshot_wormhole::SnapshotWormholeCommand;
use structopt::StructOpt;

pub fn main() -> Result<(), color_eyre::Report> {
    SnapshotWormholeCommand::from_args().exec()
}
