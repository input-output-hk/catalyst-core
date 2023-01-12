mod archive;
mod live;
mod snapshot;
mod voters;

use archive::ArchiveCommand;
use clap::Parser;
use color_eyre::Report;
use live::LiveStatsCommand;
use snapshot::SnapshotCommand;
use voters::VotersCommand;

#[derive(Parser, Debug)]
pub enum Stats {
    #[clap(subcommand)]
    Voters(VotersCommand),
    Live(LiveStatsCommand),
    Archive(ArchiveCommand),
    Snapshot(SnapshotCommand),
}

impl Stats {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Self::Voters(voters) => voters.exec(),
            Self::Live(live) => live.exec(),
            Self::Archive(archive) => archive.exec(),
            Self::Snapshot(snapshot) => snapshot.exec(),
        }
    }
}
