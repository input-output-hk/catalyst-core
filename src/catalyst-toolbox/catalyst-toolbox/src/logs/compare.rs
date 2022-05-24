use crate::logs::sentry::SentryFragmentLog;
use crate::recovery::tally::{deconstruct_account_transaction, ValidationError};

use chain_core::property::Fragment as _;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::vote::Payload;
use color_eyre::eyre::bail;
use color_eyre::Report;
use jormungandr_lib::interfaces::PersistentFragmentLog;

use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq)]
pub struct LogCmpFields {
    pub public_key: String,
    pub chain_proposal_index: u8,
    pub voteplan_id: String,
    pub choice: u8,
    pub fragment_id: String,
}

pub fn persistent_fragment_log_to_log_cmp_fields(
    fragment: &PersistentFragmentLog,
) -> Result<LogCmpFields, Report> {
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
        bail!("not vote cast transaction: {}", fragment.fragment.id())
    }
}

pub struct LogCmpStats {
    pub sentry_logs_size: usize,
    pub fragment_logs_size: usize,
    pub duplicated_sentry_logs: usize,
    pub duplicated_fragment_logs: usize,
    pub fragment_ids_differ: HashSet<String>,
    pub unhandled_fragment_logs: Vec<(Fragment, Report)>,
}

pub fn compare_logs(
    sentry_logs: &[SentryFragmentLog],
    fragment_logs: &[PersistentFragmentLog],
) -> LogCmpStats {
    let sentry_logs_size = sentry_logs.len();
    let fragment_logs_size = fragment_logs.len();
    let sentry_cmp: Vec<LogCmpFields> = sentry_logs.iter().cloned().map(Into::into).collect();

    let (fragments_cmp, unhandled_fragment_logs): (Vec<LogCmpFields>, Vec<(Fragment, Report)>) =
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
