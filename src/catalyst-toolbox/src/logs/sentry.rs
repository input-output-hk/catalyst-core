use crate::logs::compare::LogCmpFields;
use crate::recovery::tally::ValidationError;

use regex::Regex;
use reqwest::{blocking::Client, Method, Url};
use std::str::FromStr;

const REGISTERED_MESSAGE: &str = "User registered with public_key";
const MALFORMED_QR_MESSAGE: &str = "malformed encryption or decryption payload";

pub type RawLog = serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Unable to parse log from: '{0}'")]
    LogParseError(String),

    #[error("Not a vote cast transaction: {fragment_id}")]
    NotVoteCastTransaction { fragment_id: String },

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    ValidationError(#[from] ValidationError),
}

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

    pub fn get_raw_logs(&self) -> Result<String, Error> {
        self.client
            .request(Method::GET, self.api_url.clone())
            .bearer_auth(&self.auth_token)
            .send()?
            .bytes()
            .map(|b| std::str::from_utf8(&b).unwrap().to_string())
            .map_err(Error::RequestError)
    }

    pub fn get_json_logs(&self) -> Result<Vec<RawLog>, Error> {
        self.client
            .request(Method::GET, self.api_url.clone())
            .bearer_auth(&self.auth_token)
            .send()?
            .json()
            .map_err(Error::RequestError)
    }

    pub fn get_json_logs_chunks(&self, chunk: usize) -> Result<Vec<RawLog>, Error> {
        let api_url = self.api_url.join(&format!("?&cursor=0:{}:0", chunk))?;
        self.client
            .request(Method::GET, api_url)
            .bearer_auth(&self.auth_token)
            .send()?
            .json()
            .map_err(Error::RequestError)
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
    fn report(&self);
}

pub struct SuccessfulScan {
    pub total: usize,
}

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
    pub proposal_index: u8,
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

    fn report(&self) {
        println!("Total successful scans: {}", self.total);
    }
}

impl Stat for MalformedQr {
    fn check_raw_log(&mut self, log: &RawLog) {
        if raw_log_message_starts_with(log, MALFORMED_QR_MESSAGE) {
            self.total += 1;
        }
    }

    fn report(&self) {
        println!("Total malformed QR scans: {}", self.total);
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

    fn report(&self) {
        println!(
            "Total matches for [{}]: {}/{}, {}%",
            self.re.as_str(),
            self.matches,
            self.total_checked,
            (self.matches * 100) / self.total_checked
        );
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

impl Default for SuccessfulScan {
    fn default() -> Self {
        Self { total: 0 }
    }
}

impl Default for MalformedQr {
    fn default() -> Self {
        Self { total: 0 }
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

    fn report(&self) {
        match self {
            SentryLogsStatChecker::SuccessfulScans(scan) => scan.report(),
            SentryLogsStatChecker::MalformedQr(qr) => qr.report(),
            SentryLogsStatChecker::RegexMatch(re) => re.report(),
        };
    }
}

impl Stat for SentryLogsStatsExecutor {
    fn check_raw_log(&mut self, log: &RawLog) {
        for checker in self.0.iter_mut() {
            checker.check_raw_log(log);
        }
    }

    fn report(&self) {
        for checker in &self.0 {
            checker.report();
        }
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = sscanf::scanf!(
            s,
            "public_key: {} | chain proposal index: {} | proposal index: {} | voteplan: {} | choice: {} | spending counter: {} | fragment id: {}",
            String,
            u8,
            u8,
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
            .ok_or_else(|| Error::LogParseError(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::SentryFragmentLog;

    use std::str::FromStr;

    #[test]
    fn test_parse_log() {
        let _: SentryFragmentLog = SentryFragmentLog::from_str("public_key: 193cea42a72c8a4e6b4f71368f042fa072a8b5551b95ca56b68dcb368a97f78f | chain proposal index: 207 | proposal index: 238 | voteplan: ee699d301f1c6d9f9908efff4e466af0238af29e0e2df30db21a9c75d665c099 | choice: 1 | spending counter: 7 | fragment id: e747108894709e62db346d550353a62f8d410de9913e520fc3955061e4596ea7").unwrap();
    }
}
