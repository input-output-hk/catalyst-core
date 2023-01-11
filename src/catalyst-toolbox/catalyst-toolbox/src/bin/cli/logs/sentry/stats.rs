use catalyst_toolbox::logs::sentry::{
    RawLog, RegexMatch, SentryLogsStatChecker, SentryLogsStatsExecutor, Stat,
};
use color_eyre::Report;
use jcli_lib::utils::io::open_file_read;

use regex::Regex;
use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Scans {
    /// Report total successful scans
    #[clap(long)]
    scans_ok: bool,

    /// Report total malformed QRs
    #[clap(long)]
    malformed_qr: bool,
}

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Matches {
    #[clap(long, requires("re"))]
    key: Option<String>,
    #[clap(long)]
    re: Option<Regex>,
}

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Stats {
    /// Path to the input file
    #[clap(long)]
    file: PathBuf,

    /// Report all default stats, overrides single stats options
    #[clap(long)]
    all: bool,

    #[clap(flatten)]
    scans: Scans,

    #[clap(flatten)]
    matches: Matches,
}

impl Scans {
    pub fn build_checkers(&self, checkers: &mut Vec<SentryLogsStatChecker>, all: bool) {
        if self.scans_ok || all {
            checkers.push(SentryLogsStatChecker::SuccessfulScans(Default::default()));
        }
        if self.malformed_qr || all {
            checkers.push(SentryLogsStatChecker::MalformedQr(Default::default()));
        }
    }
}

impl Matches {
    pub fn build_checkers(&self, checkers: &mut Vec<SentryLogsStatChecker>, _all: bool) {
        if self.key.is_some() {
            checkers.push(SentryLogsStatChecker::RegexMatch(RegexMatch::new(
                self.re.as_ref().unwrap().clone(),
                self.key.as_ref().unwrap().clone(),
            )))
        }
    }
}

impl Stats {
    fn build_checkers(&self) -> SentryLogsStatsExecutor {
        let mut checkers = Vec::new();
        self.scans.build_checkers(&mut checkers, self.all);
        self.matches.build_checkers(&mut checkers, self.all);
        SentryLogsStatsExecutor::new(checkers)
    }

    pub fn exec(self) -> Result<(), Report> {
        let mut checker = self.build_checkers();
        let logs_reader = open_file_read(&Some(self.file))?;
        let logs: Vec<RawLog> = serde_json::from_reader(logs_reader)?;
        checker.process_raw_logs(logs.iter());
        println!("{}", checker);
        Ok(())
    }
}
