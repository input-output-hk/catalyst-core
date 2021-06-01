use crate::recovery::tally::{deconstruct_account_transaction, ValidationError};
use chain_core::property::Fragment as _;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::vote::Payload;
use jormungandr_lib::interfaces::PersistentFragmentLog;
use reqwest::{blocking::Client, Method, Url};
use std::collections::HashSet;
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

#[derive(Debug, Eq, PartialEq)]
pub struct LogCmpFields {
    pub public_key: String,
    pub chain_proposal_index: u8,
    pub voteplan_id: String,
    pub choice: u8,
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

pub fn persistent_fragment_log_to_log_cmp_fields(
    fragment: &PersistentFragmentLog,
) -> Result<LogCmpFields, Error> {
    if let Fragment::VoteCast(ref transaction) = fragment.fragment.clone() {
        let (vote_cast, identifier, choice) = deconstruct_account_transaction(
            &transaction.as_slice(),
        )
        .and_then(|(vote_cast, identifier, _)| {
            if let Payload::Public { choice } = vote_cast.payload().clone() {
                Ok((vote_cast, identifier, choice))
            } else {
                Err(ValidationError::UnsupportedPrivateVotes)
            }
        })?;
        Ok(LogCmpFields {
            fragment_id: fragment.fragment.id().to_string(),
            public_key: identifier.to_string(),
            chain_proposal_index: vote_cast.proposal_index(),
            choice: choice.as_byte(),
            voteplan_id: vote_cast.vote_plan().to_string(),
        })
    } else {
        Err(Error::NotVoteCastTransaction {
            fragment_id: fragment.fragment.id().to_string(),
        })
    }
}

pub struct LogCmpStats {
    pub sentry_logs_size: usize,
    pub fragment_logs_size: usize,
    pub duplicated_sentry_logs: usize,
    pub duplicated_fragment_logs: usize,
    pub fragment_ids_differ: HashSet<String>,
    pub unhandled_fragment_logs: Vec<(Fragment, Error)>,
}

pub fn compare_logs(
    sentry_logs: &[SentryFragmentLog],
    fragment_logs: &[PersistentFragmentLog],
) -> LogCmpStats {
    let sentry_logs_size = sentry_logs.len();
    let fragment_logs_size = fragment_logs.len();
    let sentry_cmp: Vec<LogCmpFields> = sentry_logs.iter().cloned().map(Into::into).collect();

    let (fragments_cmp, unhandled_fragment_logs): (Vec<LogCmpFields>, Vec<(Fragment, Error)>) =
        fragment_logs.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut success, mut errored), log| {
                match persistent_fragment_log_to_log_cmp_fields(log) {
                    Ok(log) => {
                        success.push(log);
                    }
                    Err(e) => errored.push((log.fragment.clone(), e)),
                };
                (success, errored)
            },
        );

    let sentry_fragments_ids: HashSet<String> = sentry_cmp
        .iter()
        .map(|e| e.fragment_id.to_string())
        .collect();
    let fragment_logs_ids: HashSet<String> = fragments_cmp
        .iter()
        .map(|e| e.fragment_id.to_string())
        .collect();
    let fragment_ids_differ: HashSet<String> = sentry_fragments_ids
        .difference(&fragment_logs_ids)
        .cloned()
        .collect();
    let duplicated_sentry_logs = sentry_logs_size - sentry_fragments_ids.len();
    let duplicated_fragment_logs = fragment_logs_size - fragment_logs_ids.len();

    LogCmpStats {
        sentry_logs_size,
        fragment_logs_size,
        duplicated_sentry_logs,
        duplicated_fragment_logs,
        fragment_ids_differ,
        unhandled_fragment_logs,
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
