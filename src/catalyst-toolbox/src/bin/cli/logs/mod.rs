mod compare;
mod sentry;

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

    #[error(transparent)]
    CompareError(#[from] compare::Error),
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Logs {
    Sentry(sentry::SentryLogs),
    Compare(compare::Compare),
}

impl Logs {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Logs::Sentry(sentry_logs) => sentry_logs.exec()?,
            Logs::Compare(compare) => compare.exec()?,
        };
        Ok(())
    }
}
