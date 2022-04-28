mod compare;
mod sentry;

use structopt::StructOpt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SentryError(#[from] sentry::Error),

    #[error(transparent)]
    CompareError(#[from] compare::Error),
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Logs {
    /// Operate over sentry logs
    Sentry(sentry::SentryLogs),
    /// Compare Sentry and Persistent fragment logs
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
