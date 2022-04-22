mod generic;

use std::str::FromStr;

use crate::{
    db::{
        models::{challenges::Challenge, proposals::FullProposalInfo},
        queries::search::generic::execute_search,
        DbConnection, DbConnectionPool,
    },
    v0::errors::HandleError,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

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
    use SearchColumn::*;

    validate_table_column_sort(table, column, sort)?;

    match table {
        SearchTable::Proposal => {
            use crate::db::views_schema::full_proposals_info::dsl::*;
            use full_proposals_info as proposals;

            let mut items = match column {
                ProposalTitle => execute_search(proposals, proposal_title, &search, conn),
                ProposalAuthor => execute_search(proposals, proposer_name, &search, conn),
                ProposalSummary => execute_search(proposals, proposal_summary, &search, conn),
                _ => unreachable!(),
            }?;

            sort_proposals(&mut items, sort);
            Ok(SearchItems::Proposal(items))
        }
        SearchTable::Challenge => {
            use crate::db::schema::challenges::dsl::*;

            let mut items = match column {
                ChallengeTitle => execute_search(challenges, title, &search, conn),
                ChallengeType => execute_search(challenges, challenge_type, &search, conn),
                ChallengeDesc => execute_search(challenges, description, &search, conn),
                _ => unreachable!(),
            }?;

            sort_challenges(&mut items, sort);
            Ok(SearchItems::Challenge(items))
        }
    }
}

fn sort_proposals(items: &mut [FullProposalInfo], sort: Option<SearchSort>) {
    match sort {
        None => {}
        Some(SearchSort::Random) => items.shuffle(&mut thread_rng()),
        Some(SearchSort::ProposalFunds) => items.sort_unstable_by(|a, b| {
            Ord::cmp(&a.proposal.proposal_funds, &b.proposal.proposal_funds)
        }),
        Some(SearchSort::ProposalTitle) => items.sort_unstable_by(|a, b| {
            Ord::cmp(&a.proposal.proposal_title, &b.proposal.proposal_title)
        }),
        Some(SearchSort::ProposalAdvisor) => {
            items.sort_unstable_by(|a, b| {
                Ord::cmp(
                    &a.proposal.proposer.proposer_name,
                    &b.proposal.proposer.proposer_name,
                )
            });
        }
        Some(SearchSort::ChallengeTitle) => unreachable!(),
    }
}

fn sort_challenges(items: &mut [Challenge], sort: Option<SearchSort>) {
    match sort {
        None => {}
        Some(SearchSort::Random) => items.shuffle(&mut thread_rng()),
        Some(SearchSort::ChallengeTitle) => items.sort_unstable_by(|a, b| a.title.cmp(&b.title)),
        Some(SearchSort::ProposalFunds)
        | Some(SearchSort::ProposalTitle)
        | Some(SearchSort::ProposalAdvisor) => unreachable!(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchItems {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum SearchTable {
    Challenge,
    Proposal,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum SearchColumn {
    ChallengeTitle,
    ChallengeType,
    ChallengeDesc,
    ProposalAuthor,
    ProposalTitle,
    ProposalSummary,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum SearchSort {
    ChallengeTitle,
    ProposalFunds,
    ProposalAdvisor,
    ProposalTitle,
    Random,
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
