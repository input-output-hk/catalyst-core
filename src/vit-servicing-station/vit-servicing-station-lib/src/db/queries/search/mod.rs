mod generic;

use std::str::FromStr;

use crate::{
    db::{
        models::{challenges::Challenge, proposals::FullProposalInfo},
        DbConnection, DbConnectionPool,
    },
    db_search,
    v0::errors::HandleError,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use self::generic::search_query;

pub async fn search_db(
    table: SearchTable,
    column: SearchColumn,
    sort: Option<SearchSort>,
    search: String,
    pool: &DbConnectionPool,
) -> Result<SearchItems, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search_sync(table, column, sort, search, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

fn search_sync(
    table: SearchTable,
    column: SearchColumn,
    sort: Option<SearchSort>,
    search: String,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<SearchItems, HandleError> {
    validate_table_column_sort(table, column, sort)?;

    match table {
        SearchTable::Proposal => {
            use crate::db::views_schema::full_proposals_info::dsl::*;

            let mut items = match column {
                SearchColumn::ProposalTitle => {
                    db_search!(conn => search in full_proposals_info.proposal_title)
                }
                SearchColumn::ProposalSummary => {
                    db_search!(conn => search in full_proposals_info.proposal_summary)
                }
                SearchColumn::ProposalAuthor => {
                    db_search!(conn => search in full_proposals_info.proposer_name)
                }
                _ => unreachable!(),
            }
            .map_err(|_| HandleError::InternalError("error searching".to_string()))?;

            sort_proposals(&mut items, sort);

            Ok(SearchItems::Proposal(items))
        }
        SearchTable::Challenge => {
            use crate::db::schema::challenges::dsl::*;

            let mut items = match column {
                SearchColumn::ChallengeTitle => {
                    db_search!(conn => search in challenges.title)
                }
                SearchColumn::ChallengeType => {
                    db_search!(conn => search in challenges.challenge_type)
                }
                SearchColumn::ChallengeDesc => {
                    db_search!(conn => search in challenges.description)
                }
                _ => unreachable!(),
            }
            .map_err(|_| HandleError::InternalError("error searching".to_string()))?;

            sort_challenges(&mut items, sort);

            Ok(SearchItems::Challenge(items))
        }
    }
}

fn sort_proposals(items: &mut [FullProposalInfo], sort: Option<SearchSort>) {
    match sort {
        None => {}
        Some(SearchSort::Random) => items.shuffle(&mut thread_rng()),
        Some(SearchSort::ProposalFunds) => {
            items.sort_by(|a, b| a.proposal.proposal_funds.cmp(&b.proposal.proposal_funds));
        }
        Some(SearchSort::ProposalTitle) => {
            items.sort_by(|a, b| a.proposal.proposal_title.cmp(&b.proposal.proposal_title));
        }
        Some(SearchSort::ProposalAdvisor) => {
            items.sort_by(|a, b| {
                a.proposal
                    .proposer
                    .proposer_name
                    .cmp(&b.proposal.proposer.proposer_name)
            });
        }
        Some(SearchSort::ChallengeTitle) => unreachable!(),
    }
}

fn sort_challenges(items: &mut [Challenge], sort: Option<SearchSort>) {
    match sort {
        None => {}
        Some(SearchSort::Random) => items.shuffle(&mut thread_rng()),
        Some(SearchSort::ChallengeTitle) => {
            items.sort_by(|a, b| a.title.cmp(&b.title));
        }
        Some(SearchSort::ProposalFunds)
        | Some(SearchSort::ProposalTitle)
        | Some(SearchSort::ProposalAdvisor) => unreachable!(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum SearchItems {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SearchTable {
    Challenge,
    Proposal,
}

impl FromStr for SearchTable {
    type Err = HandleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "challenge" => Ok(Self::Challenge),
            "proposal" => Ok(Self::Proposal),
            s => Err(HandleError::BadRequest(format!("unknown resource: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SearchColumn {
    ChallengeTitle,
    ChallengeType,
    ChallengeDesc,
    ProposalAuthor,
    ProposalTitle,
    ProposalSummary,
}

impl FromStr for SearchColumn {
    type Err = HandleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "challenge_title" => Ok(Self::ChallengeTitle),
            "challenge_type" => Ok(Self::ChallengeType),
            "challenge_desc" => Ok(Self::ChallengeDesc),
            "proposal_author" => Ok(Self::ProposalAuthor),
            "proposal_title" => Ok(Self::ProposalAuthor),
            "proposal_summary" => Ok(Self::ProposalSummary),
            s => Err(HandleError::BadRequest(format!("unknown column: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SearchSort {
    ChallengeTitle,
    ProposalFunds,
    ProposalAdvisor,
    ProposalTitle,
    Random,
}

impl FromStr for SearchSort {
    type Err = HandleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "challenge_title" => Ok(Self::ChallengeTitle),
            "proposal_funds" => Ok(Self::ProposalFunds),
            "proposal_advisor" => Ok(Self::ProposalAdvisor),
            "proposal_title" => Ok(Self::ProposalTitle),
            "random" => Ok(Self::Random),
            s => Err(HandleError::BadRequest(format!("unknown column: {s}"))),
        }
    }
}

fn validate_table_column_sort(
    table: SearchTable,
    column: SearchColumn,
    sort: Option<SearchSort>,
) -> Result<(), HandleError> {
    use SearchColumn::*;
    use SearchTable::*;

    match (table, column) {
        (Challenge, ChallengeTitle | ChallengeType | ChallengeDesc) => {}
        (Proposal, ProposalAuthor | ProposalTitle | ProposalSummary) => {}
        _ => {
            return Err(HandleError::BadRequest(format!(
                "cannot query column {column:?} on table {table:?}"
            )))
        }
    };

    // new scope to avoid name collisions with column types
    {
        use SearchSort::*;
        use SearchTable::*;
        match (table, sort) {
            (_, None | Some(Random)) => {}
            (Challenge, Some(ChallengeTitle)) => {}
            (Proposal, Some(ProposalFunds | ProposalTitle | ProposalAdvisor)) => {}
            _ => {
                return Err(HandleError::BadRequest(format!(
                    "cannot sort table {table:?} by column {column:?}"
                )))
            }
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_ok_with_good_column_and_sort() {
        validate_table_column_sort(
            SearchTable::Proposal,
            SearchColumn::ProposalTitle,
            Some(SearchSort::ProposalAdvisor),
        )
        .unwrap();
    }

    #[test]
    fn should_be_err_with_mismatching_column_or_sort() {
        validate_table_column_sort(SearchTable::Proposal, SearchColumn::ChallengeType, None)
            .unwrap_err();

        validate_table_column_sort(
            SearchTable::Proposal,
            SearchColumn::ProposalTitle,
            Some(SearchSort::ChallengeTitle),
        )
        .unwrap_err();
    }
}
