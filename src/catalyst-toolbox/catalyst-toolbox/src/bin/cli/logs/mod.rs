mod compare;
mod sentry;

use color_eyre::Report;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Logs {
    /// Operate over sentry logs
    Sentry(sentry::SentryLogs),
    /// Compare Sentry and Persistent fragment logs
    Compare(compare::Compare),
}

impl Logs {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Logs::Sentry(sentry_logs) => sentry_logs.exec()?,
            Logs::Compare(compare) => compare.exec()?,
        };
        Ok(())
    }
}
