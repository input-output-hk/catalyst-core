mod compare;
mod sentry;

use clap::Parser;
use color_eyre::Report;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Logs {
    /// Operate over sentry logs
    #[clap(subcommand)]
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
