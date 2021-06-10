use super::Error;
use catalyst_toolbox::logs::sentry::{
    RawLog, SentryLogsStatChecker, SentryLogsStatsExecutor, Stat,
};
use jcli_lib::utils::io::open_file_read;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Stats {
    /// Path to the input file
    #[structopt(long)]
    file: PathBuf,

    /// Report all default stats, overrides single stats options
    #[structopt(long)]
    all: bool,

    /// Report total successful scans
    #[structopt(long)]
    scans_ok: bool,

    /// Report total malformed QRs
    #[structopt(long)]
    malformed_qr: bool,
}

impl Stats {
    pub fn exec(self) -> Result<(), Error> {
        let mut checker = build_checkers(&self);
        let logs_reader = open_file_read(&Some(self.file))?;
        let logs: Vec<RawLog> = serde_json::from_reader(logs_reader)?;
        check_stats(&mut checker, &logs);
        checker.report();
        Ok(())
    }
}

fn build_checkers(stats: &Stats) -> SentryLogsStatsExecutor {
    let checkers = if stats.all {
        vec![
            SentryLogsStatChecker::SuccessfulScans(Default::default()),
            SentryLogsStatChecker::MalformedQr(Default::default()),
        ]
    } else {
        let mut checkers = Vec::new();
        if stats.scans_ok {
            checkers.push(SentryLogsStatChecker::SuccessfulScans(Default::default()));
        }
        if stats.malformed_qr {
            checkers.push(SentryLogsStatChecker::MalformedQr(Default::default()));
        }
        checkers
    };
    SentryLogsStatsExecutor::new(checkers)
}

fn check_stats(executor: &mut SentryLogsStatsExecutor, logs: &[RawLog]) {
    for log in logs {
        executor.check_raw_log(log);
    }
}
