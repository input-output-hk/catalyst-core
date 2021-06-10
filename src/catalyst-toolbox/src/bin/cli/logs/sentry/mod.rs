mod download;
mod stats;

use catalyst_toolbox::logs::sentry::Error as SentryLogError;

use structopt::StructOpt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SentryLogError(#[from] SentryLogError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SentryLogs {
    /// Download logs from sentry
    Download(download::Download),
    /// Stats report about logs
    Stats(stats::Stats),
}

impl SentryLogs {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            SentryLogs::Download(download) => download.exec(),
            SentryLogs::Stats(stats) => stats.exec(),
        }
    }
}
