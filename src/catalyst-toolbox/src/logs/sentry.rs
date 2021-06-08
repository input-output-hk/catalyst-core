use crate::logs::compare::LogCmpFields;
use crate::recovery::tally::ValidationError;

use reqwest::{blocking::Client, Method, Url};
use std::str::FromStr;

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
        println!("{}", api_url);
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
