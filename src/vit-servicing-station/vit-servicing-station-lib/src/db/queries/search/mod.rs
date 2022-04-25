mod generic;

use crate::{
    db::{
        queries::search::generic::{search, RANDOM},
        DbConnection, DbConnectionPool,
    },
    v0::{endpoints::search::requests::*, errors::HandleError},
};
use diesel::r2d2::{ConnectionManager, PooledConnection};

pub async fn search_db(
    SearchRequest {
        table,
        column,
        sort,
        query,
    }: SearchRequest,
    pool: &DbConnectionPool,
) -> Result<SearchResponse, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        validate_table_column_sort(table, column, sort)?;
        match table {
            SearchTable::Proposal => search_proposals(column, sort, &query, &db_conn),
            SearchTable::Challenge => search_challenges(column, sort, &query, &db_conn),
        }
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

fn search_proposals(
    column: SearchColumn,
    sort: SearchSort,
    query: &str,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<SearchResponse, HandleError> {
    use crate::db::views_schema::full_proposals_info::dsl::*;
    use full_proposals_info as proposals;

    let vec = match column {
        SearchColumn::ProposalTitle => match sort {
            SearchSort::Index => search(proposals, proposal_title, id, query, conn),
            SearchSort::Random => search(proposals, proposal_title, RANDOM, query, conn),
            SearchSort::ProposalFunds => {
                search(proposals, proposal_title, proposal_funds, query, conn)
            }
            SearchSort::ProposalAdvisor => {
                search(proposals, proposal_title, proposer_name, query, conn)
            }
            SearchSort::ProposalTitle => {
                search(proposals, proposal_title, proposal_title, query, conn)
            }
            _ => unreachable!(),
        },
        SearchColumn::ProposalAuthor => match sort {
            SearchSort::Index => search(proposals, proposer_name, id, query, conn),
            SearchSort::Random => search(proposals, proposer_name, RANDOM, query, conn),
            SearchSort::ProposalFunds => {
                search(proposals, proposer_name, proposal_funds, query, conn)
            }
            SearchSort::ProposalAdvisor => {
                search(proposals, proposer_name, proposer_name, query, conn)
            }
            SearchSort::ProposalTitle => {
                search(proposals, proposer_name, proposal_title, query, conn)
            }
            _ => unreachable!(),
        },
        SearchColumn::ProposalSummary => match sort {
            SearchSort::Index => search(proposals, proposal_summary, id, query, conn),
            SearchSort::Random => search(proposals, proposal_summary, RANDOM, query, conn),
            SearchSort::ProposalFunds => {
                search(proposals, proposal_summary, proposal_funds, query, conn)
            }
            SearchSort::ProposalAdvisor => {
                search(proposals, proposal_summary, proposer_name, query, conn)
            }
            SearchSort::ProposalTitle => {
                search(proposals, proposal_summary, proposal_title, query, conn)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }?;
    Ok(SearchResponse::Proposal(vec))
}

fn search_challenges(
    column: SearchColumn,
    sort: SearchSort,
    query: &str,
    conn: &PooledConnection<ConnectionManager<DbConnection>>,
) -> Result<SearchResponse, HandleError> {
    use crate::db::schema::challenges::dsl::*;

    let vec = match column {
        SearchColumn::ChallengeType => match sort {
            SearchSort::Index => search(challenges, challenge_type, id, query, conn),
            SearchSort::Random => search(challenges, challenge_type, RANDOM, query, conn),
            SearchSort::ChallengeTitle => search(challenges, challenge_type, title, query, conn),
            _ => unreachable!(),
        },
        SearchColumn::ChallengeTitle => match sort {
            SearchSort::Index => search(challenges, title, id, query, conn),
            SearchSort::Random => search(challenges, title, RANDOM, query, conn),
            SearchSort::ChallengeTitle => search(challenges, title, title, query, conn),
            _ => unreachable!(),
        },
        SearchColumn::ChallengeDesc => match sort {
            SearchSort::Index => search(challenges, description, id, query, conn),
            SearchSort::Random => search(challenges, description, RANDOM, query, conn),
            SearchSort::ChallengeTitle => search(challenges, description, title, query, conn),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }?;
    Ok(SearchResponse::Challenge(vec))
}

fn validate_table_column_sort(
    table: SearchTable,
    column: SearchColumn,
    sort: SearchSort,
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
            (_, Index | Random) => {}
            (Challenge, ChallengeTitle) => {}
            (Proposal, ProposalFunds | ProposalTitle | ProposalAdvisor) => {}
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
            SearchSort::ProposalAdvisor,
        )
        .unwrap();
    }

    #[test]
    fn should_be_err_with_mismatching_column_or_sort() {
        validate_table_column_sort(
            SearchTable::Proposal,
            SearchColumn::ChallengeType,
            SearchSort::Index,
        )
        .unwrap_err();

        validate_table_column_sort(
            SearchTable::Proposal,
            SearchColumn::ProposalTitle,
            SearchSort::ChallengeTitle,
        )
        .unwrap_err();
    }
}
