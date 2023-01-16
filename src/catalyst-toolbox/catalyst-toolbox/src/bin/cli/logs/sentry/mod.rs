mod download;
mod stats;

use clap::Parser;
use color_eyre::Report;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub enum SentryLogs {
    /// Download logs from sentry
    Download(download::Download),
    /// Stats report about logs
    Stats(stats::Stats),
}

impl SentryLogs {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            SentryLogs::Download(download) => download.exec(),
            SentryLogs::Stats(stats) => stats.exec(),
        }
    }
}
