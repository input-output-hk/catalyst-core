use crate::logs::compare::LogCmpFields;

use color_eyre::Report;
use regex::Regex;
use reqwest::{blocking::Client, Method, Url};

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

const REGISTERED_MESSAGE: &str = "User registered with public_key";
const MALFORMED_QR_MESSAGE: &str = "malformed encryption or decryption payload";

pub type RawLog = serde_json::Value;

pub struct SentryLogClient {
    client: Client,
    api_url: Url,
    auth_token: String,
}

impl SentryLogClient {
    pub fn new(api_url: Url, auth_token: String) -> Self {
        let client = Client::new();
        Self {
            client,
            api_url,
            auth_token,
        }
    }

    pub fn get_raw_logs(&self) -> Result<String, Report> {
        Ok(self
            .client
            .request(Method::GET, self.api_url.clone())
            .bearer_auth(&self.auth_token)
            .send()?
            .bytes()
            .map(|b| std::str::from_utf8(&b).unwrap().to_string())?)
    }

    pub fn get_json_logs(&self) -> Result<Vec<RawLog>, Report> {
        Ok(self
            .client
            .request(Method::GET, self.api_url.clone())
            .bearer_auth(&self.auth_token)
            .send()?
            .json()?)
    }

    pub fn get_json_logs_chunks(&self, chunk: usize) -> Result<Vec<RawLog>, Report> {
        let api_url = self.api_url.join(&format!("?&cursor=0:{}:0", chunk))?;
        Ok(self
            .client
            .request(Method::GET, api_url)
            .bearer_auth(&self.auth_token)
            .send()?
            .json()?)
    }
}

pub struct LazySentryLogs {
    client: SentryLogClient,
    chunk_size: usize,
}

impl LazySentryLogs {
    pub fn new(client: SentryLogClient, chunk_size: usize) -> Self {
        Self { client, chunk_size }
    }
}

impl IntoIterator for LazySentryLogs {
    type Item = RawLog;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            (0..)
                .map(move |i| {
                    self.client
                        .get_json_logs_chunks(i * self.chunk_size)
                        .ok()
                        .and_then(|v| if v.is_empty() { None } else { Some(v) })
                })
                .take_while(Option::is_some)
                .flat_map(Option::unwrap),
        )
    }
}

pub trait Stat {
    fn check_raw_log(&mut self, log: &RawLog);
    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result;
    fn process_raw_logs<'l>(&mut self, logs: impl Iterator<Item = &'l RawLog>) {
        for log in logs {
            self.check_raw_log(log);
        }
    }
}

#[derive(Default)]
pub struct SuccessfulScan {
    pub total: usize,
}

#[derive(Default)]
pub struct MalformedQr {
    pub total: usize,
}

pub struct RegexMatch {
    key: String,
    re: Regex,
    total_checked: usize,
    pub matches: usize,
}

pub enum SentryLogsStatChecker {
    SuccessfulScans(SuccessfulScan),
    MalformedQr(MalformedQr),
    RegexMatch(RegexMatch),
}

pub struct SentryLogsStatsExecutor(Vec<SentryLogsStatChecker>);

#[derive(Debug, Clone)]
pub struct SentryFragmentLog {
    pub public_key: String,
    pub chain_proposal_index: u8,
    pub proposal_index: u32,
    pub voteplan_id: String,
    pub choice: u8,
    pub spending_counter: u64,
    pub fragment_id: String,
}

fn raw_log_message_starts_with(log: &RawLog, pattern: &str) -> bool {
    log.get("message")
        .and_then(|message| message.as_str().map(|s| s.starts_with(pattern)))
        .unwrap_or(false)
}

impl Stat for SuccessfulScan {
    fn check_raw_log(&mut self, log: &RawLog) {
        if raw_log_message_starts_with(log, REGISTERED_MESSAGE) {
            self.total += 1;
        }
    }

    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "Total successful scans: {}", self.total)
    }
}

impl Stat for MalformedQr {
    fn check_raw_log(&mut self, log: &RawLog) {
        if raw_log_message_starts_with(log, MALFORMED_QR_MESSAGE) {
            self.total += 1;
        }
    }

    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "Total malformed QR scans: {}", self.total)
    }
}

impl Stat for RegexMatch {
    fn check_raw_log(&mut self, log: &RawLog) {
        self.total_checked += 1;
        if let Some(entry) = log.get(&self.key).and_then(|value| value.as_str()) {
            if self.re.is_match(entry) {
                self.matches += 1;
            }
        }
    }

    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Total matches for [{}]: {}/{}, {}%",
            self.re.as_str(),
            self.matches,
            self.total_checked,
            (self.matches * 100) / self.total_checked,
        )
    }
}

impl SuccessfulScan {
    pub fn new() -> Self {
        Default::default()
    }
}

impl MalformedQr {
    pub fn new() -> Self {
        Default::default()
    }
}

impl RegexMatch {
    pub fn new(re: Regex, key: String) -> Self {
        Self {
            re,
            key,
            total_checked: 0,
            matches: 0,
        }
    }
}

impl Stat for SentryLogsStatChecker {
    fn check_raw_log(&mut self, log: &RawLog) {
        match self {
            SentryLogsStatChecker::SuccessfulScans(scan) => scan.check_raw_log(log),
            SentryLogsStatChecker::MalformedQr(qr) => qr.check_raw_log(log),
            SentryLogsStatChecker::RegexMatch(re) => re.check_raw_log(log),
        };
    }

    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            SentryLogsStatChecker::SuccessfulScans(scan) => scan.report(formatter),
            SentryLogsStatChecker::MalformedQr(qr) => qr.report(formatter),
            SentryLogsStatChecker::RegexMatch(re) => re.report(formatter),
        }
    }
}

impl Stat for SentryLogsStatsExecutor {
    fn check_raw_log(&mut self, log: &RawLog) {
        for checker in self.0.iter_mut() {
            checker.check_raw_log(log);
        }
    }

    fn report(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        for checker in &self.0 {
            checker.report(formatter)?;
            writeln!(formatter)?;
        }
        Ok(())
    }
}

impl Display for SentryLogsStatsExecutor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.report(f)
    }
}

impl SentryLogsStatsExecutor {
    pub fn new(checkers: Vec<SentryLogsStatChecker>) -> Self {
        Self(checkers)
    }
}

impl From<SentryFragmentLog> for LogCmpFields {
    fn from(log: SentryFragmentLog) -> Self {
        Self {
            public_key: log.public_key,
            chain_proposal_index: log.chain_proposal_index,
            voteplan_id: log.voteplan_id,
            choice: log.choice,
            fragment_id: log.fragment_id,
        }
    }
}

impl FromStr for SentryFragmentLog {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = sscanf::scanf!(
            s,
            "public_key: {} | chain proposal index: {} | proposal index: {} | voteplan: {} | choice: {} | spending counter: {} | fragment id: {}",
            String,
            u8,
            u32,
            String,
            u8,
            u64,
            String
        );
        parsed
            .map(
                |(
                    public_key,
                    chain_proposal_index,
                    proposal_index,
                    voteplan_id,
                    choice,
                    spending_counter,
                    fragment_id,
                )| Self {
                    public_key,
                    chain_proposal_index,
                    proposal_index,
                    voteplan_id,
                    choice,
                    spending_counter,
                    fragment_id,
                },
            )
            .ok_or_else(|| color_eyre::eyre::eyre!("log parse error: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::SentryFragmentLog;
    use crate::logs::sentry::{
        MalformedQr, RawLog, RegexMatch, SentryLogsStatChecker, SentryLogsStatsExecutor, Stat,
        SuccessfulScan, MALFORMED_QR_MESSAGE, REGISTERED_MESSAGE,
    };

    use std::str::FromStr;

    use regex::Regex;

    fn generate_test_raw_log_set(success: usize, unssucess: usize) -> Vec<RawLog> {
        let successful_scan_log = serde_json::json!({ "message": REGISTERED_MESSAGE });
        let unsuccessful_scan_log = serde_json::json!({ "message": MALFORMED_QR_MESSAGE });

        let success_logs = (0..success)
            .into_iter()
            .map(|_| successful_scan_log.clone());

        let unsuccessful_logs = (0..unssucess)
            .into_iter()
            .map(|_| unsuccessful_scan_log.clone());

        success_logs.chain(unsuccessful_logs).collect()
    }

    #[test]
    fn test_parse_log() {
        let _: SentryFragmentLog = SentryFragmentLog::from_str("public_key: 7013d6d804d141b89591936a1a4892f498a8901d3c20487ed130f96f9b8446e1 | chain proposal index: 247 | proposal index: 272 | voteplan: aed19eed570c715c847739aefdcf72c5bd2b51c89ddca26f653fb33f83247481 | choice: 1 | spending counter: 18 | fragment id: 43cd7b5f91c257aba6ae114f28f1acd9bb0aa5547ae11f29f7c33b93eddefe37").unwrap();
    }

    #[test]
    fn test_successful_scan() {
        let logs = generate_test_raw_log_set(10, 3);
        let mut checker = SuccessfulScan::new();
        checker.process_raw_logs(logs.iter());
        assert_eq!(checker.total, 10);
    }

    #[test]
    fn test_unsuccessful_scan() {
        let logs = generate_test_raw_log_set(3, 10);
        let mut checker = MalformedQr::new();
        checker.process_raw_logs(logs.iter());
        assert_eq!(checker.total, 10);
    }

    #[test]
    fn test_regex_match_scan() {
        let logs = generate_test_raw_log_set(10, 15);
        let mut checker = RegexMatch::new(
            Regex::from_str("User registered with public_key").unwrap(),
            "message".to_string(),
        );
        checker.process_raw_logs(logs.iter());
        assert_eq!(checker.total_checked, 25);
        assert_eq!(checker.matches, 10);
    }

    #[test]
    fn test_executor_scan() {
        let total_success = 10;
        let total_malformed = 15;
        let logs = generate_test_raw_log_set(total_success, total_malformed);
        let re = SentryLogsStatChecker::RegexMatch(RegexMatch::new(
            Regex::from_str("User registered with public_key").unwrap(),
            "message".to_string(),
        ));
        let success = SentryLogsStatChecker::SuccessfulScans(Default::default());
        let qr = SentryLogsStatChecker::MalformedQr(Default::default());

        let mut checker = SentryLogsStatsExecutor::new(vec![success, qr, re]);

        checker.process_raw_logs(logs.iter());
        for inner in &checker.0 {
            match inner {
                SentryLogsStatChecker::SuccessfulScans(ss) => {
                    assert_eq!(ss.total, total_success);
                }
                SentryLogsStatChecker::MalformedQr(qr) => {
                    assert_eq!(qr.total, total_malformed);
                }
                SentryLogsStatChecker::RegexMatch(re) => {
                    assert_eq!(re.total_checked, total_malformed + total_success);
                    assert_eq!(re.matches, total_success);
                }
            }
        }
    }
}
