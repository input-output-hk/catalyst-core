mod archive;
mod live;
mod snapshot;
mod voters;

use archive::ArchiveCommand;
use color_eyre::Report;
use live::LiveStatsCommand;
use snapshot::SnapshotCommand;
use structopt::StructOpt;
use voters::VotersCommand;

#[derive(StructOpt, Debug)]
pub enum Stats {
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
