mod advisor_reviews;
mod archive;
mod ideascale;
mod kedqr;
mod logs;
mod notifications;
mod proposal_score;
mod recovery;
mod rewards;
mod snapshot;
mod stats;
mod sve_snapshot;
mod vote_check;

use clap::Parser;
use color_eyre::Report;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Cli {
    /// display full version details (software version, source version, targets and compiler used)
    #[clap(long = "full-version")]
    full_version: bool,

    /// display the sources version, allowing to check the source's hash used to compile this executable.
    /// this option is useful for scripting retrieving the logs of the version of this application.
    #[clap(long = "source-version")]
    source_version: bool,

    #[clap(subcommand)]
    command: Option<CatalystCommand>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum CatalystCommand {
    /// Proposal score operations
    ProposalScore(proposal_score::ProposalScore),
    /// Rewards related operations
    #[clap(subcommand)]
    Rewards(rewards::Rewards),
    /// Send push notification to pushwoosh service
    #[clap(subcommand)]
    Push(notifications::PushNotifications),
    /// Tally recovery utility
    #[clap(subcommand)]
    Recover(recovery::Recover),
    /// Download, compare and get stats from sentry and persistent fragment logs
    #[clap(subcommand)]
    Logs(logs::Logs),
    /// Generate qr codes
    #[clap(subcommand)]
    QrCode(kedqr::QrCodeCmd),
    /// Interact with the Ideascale API
    #[clap(subcommand)]
    Ideascale(ideascale::Ideascale),
    /// Advisor reviews related operations
    #[clap(subcommand)]
    Reviews(advisor_reviews::Reviews),
    /// Dump information related to catalyst fund
    #[clap(subcommand)]
    Archive(archive::Archive),
    /// Validate catalyst elections
    VoteCheck(vote_check::VoteCheck),
    /// Prints voting statistics
    #[clap(subcommand)]
    Stats(stats::Stats),
    /// Process raw registrations to produce initial blockchain setup
    Snapshot(snapshot::SnapshotCmd),
    /// Process raw registrations for SVE.
    SveSnapshot(sve_snapshot::SveSnapshotCmd),
}

impl Cli {
    pub fn exec(self) -> Result<(), Report> {
        use std::io::Write as _;
        if self.full_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("FULL_VERSION"))?)
        } else if self.source_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("SOURCE_VERSION"))?)
        } else if let Some(cmd) = self.command {
            cmd.exec()
        } else {
            writeln!(std::io::stderr(), "No command, try `--help'")?;
            std::process::exit(1);
        }
    }
}

impl CatalystCommand {
    pub fn exec(self) -> Result<(), Report> {
        use self::CatalystCommand::*;
        match self {
            ProposalScore(proposal_score) => proposal_score.exec()?,
            Rewards(rewards) => rewards.exec()?,
            Push(notifications) => notifications.exec()?,
            Recover(recover) => recover.exec()?,
            Logs(logs) => logs.exec()?,
            QrCode(kedqr) => kedqr.exec()?,
            Ideascale(ideascale) => ideascale.exec()?,
            Reviews(reviews) => reviews.exec()?,
            Archive(archive) => archive.exec()?,
            VoteCheck(vote_check) => vote_check.exec()?,
            Stats(stats) => stats.exec()?,
            Snapshot(snapshot) => snapshot.exec()?,
            SveSnapshot(sve_snapshot) => sve_snapshot.exec()?,
        };
        Ok(())
    }
}
