use std::convert::TryInto;

use crate::{
    db::{schema, DbConnection, DbConnectionPool},
    v0::{
        endpoints::search::requests::{
            Column, Constraint, OrderBy, SearchCountQuery, SearchQuery, SearchResponse, Table,
        },
        errors::HandleError,
    },
};
use diesel::{
    backend::Backend, expression_methods::ExpressionMethods, QueryDsl, RunQueryDsl,
    TextExpressionMethods,
};

pub async fn search_db(
    query: SearchQuery,
    pool: &DbConnectionPool,
) -> Result<SearchResponse, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search(query, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn search_count_db(
    query: SearchCountQuery,
    pool: &DbConnectionPool,
) -> Result<i64, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search_count(query, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

no_arg_sql_function!(RANDOM, ());

type ChallengesSelectST = (
    diesel::sql_types::Integer,
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::BigInt,
    diesel::sql_types::BigInt,
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
);

fn build_challenges_query<'a, DB: 'a + Backend>(
    filter: Vec<Constraint>,
    order_by: Vec<OrderBy>,
) -> Result<
    diesel::query_builder::BoxedSelectStatement<
        'a,
        ChallengesSelectST,
        schema::challenges::table,
        DB,
    >,
    HandleError,
> {
    use crate::db::schema::challenges::dsl::*;
    use Column::*;

    let mut query = challenges.into_boxed();

    for constraint in filter {
        match constraint {
            Constraint::Text { search, column } => {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(title.like(search)),
                    Desc => query.filter(description.like(search)),
                    Type => query.filter(challenge_type.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }
            Constraint::Range { .. } => {
                return Err(HandleError::BadRequest("invalid constraint".to_string()));
            }
        }
    }

    for order in order_by {
        query = match order {
            OrderBy::Random => query.then_order_by(RANDOM),
            OrderBy::Column { column, descending } => match (descending, column) {
                (false, Title) => query.then_order_by(title),
                (false, Desc) => query.then_order_by(description),
                (false, Type) => query.then_order_by(challenge_type),
                (true, Title) => query.then_order_by(title.desc()),
                (true, Desc) => query.then_order_by(description.desc()),
                (true, Type) => query.then_order_by(challenge_type.desc()),
                _ => return Err(HandleError::BadRequest("invalid column".to_string())),
            },
        }
    }
    Ok(query)
}

type SelectProposalsST = (
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::BigInt,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::BigInt,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Binary,
    diesel::sql_types::Text,
    diesel::sql_types::Integer,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Integer,
    diesel::sql_types::BigInt,
    diesel::sql_types::BigInt,
    diesel::sql_types::BigInt,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::Nullable<diesel::sql_types::Text>,
    diesel::sql_types::BigInt,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
);

fn build_proposals_query<'a, DB: 'a + Backend>(
    filter: Vec<Constraint>,
    order_by: Vec<OrderBy>,
) -> Result<
    diesel::query_builder::BoxedSelectStatement<
        'a,
        SelectProposalsST,
        crate::db::views_schema::full_proposals_info::table,
        DB,
    >,
    HandleError,
> {
    use crate::db::views_schema::full_proposals_info::dsl::*;
    use full_proposals_info as proposals;
    use Column::*;

    let mut query = proposals.into_boxed();

    for constraint in filter {
        match constraint {
            Constraint::Text { search, column } => {
                let search = format!("%{search}%");
                query = match column {
                    Title => query.filter(proposal_title.like(search)),
                    Desc => query.filter(proposal_summary.like(search)),
                    Author => query.filter(proposer_name.like(search)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                }
            }
            Constraint::Range {
                lower,
                upper,
                column,
            } => {
                let lower = lower.unwrap_or(i64::MIN);
                let upper = upper.unwrap_or(i64::MAX);
                query = match column {
                    ImpactScore => query
                        .filter(proposal_impact_score.ge(lower))
                        .filter(proposal_impact_score.le(upper)),
                    Funds => query
                        .filter(proposal_funds.ge(lower))
                        .filter(proposal_funds.le(upper)),
                    _ => return Err(HandleError::BadRequest("invalid column".to_string())),
                };
            }
        }
    }

    for order in order_by {
        query = match order {
            OrderBy::Random => query.then_order_by(RANDOM),
            OrderBy::Column { column, descending } => match (descending, column) {
                (false, Title) => query.then_order_by(proposal_title),
                (false, Desc) => query.then_order_by(proposal_summary),
                (false, Author) => query.then_order_by(proposer_name),
                (false, Funds) => query.then_order_by(proposal_funds),
                (true, Title) => query.then_order_by(proposal_title.desc()),
                (true, Desc) => query.then_order_by(proposal_summary.desc()),
                (true, Author) => query.then_order_by(proposer_name.desc()),
                (true, Funds) => query.then_order_by(proposal_funds.desc()),
                _ => return Err(HandleError::BadRequest("invalid column".to_string())),
            },
        }
    }
    Ok(query)
}

fn search(
    SearchQuery {
        query,
        limit,
        offset,
    }: SearchQuery,
    conn: &DbConnection,
) -> Result<SearchResponse, HandleError> {
    let SearchCountQuery {
        table,
        filter,
        order_by,
    } = query;

    match table {
        Table::Challenges => {
            let vec = {
                let mut query = build_challenges_query(filter, order_by)?;

                if let Some(limit) = limit {
                    query = query.limit(map_limit(limit)?);
                }

                if let Some(offset) = offset {
                    query = query.offset(map_offset(offset)?);
                }

                query.load(conn)
            }
            .map_err(|_| HandleError::InternalError("error searching".to_string()))?;

            Ok(SearchResponse::Challenge(vec))
        }
        Table::Proposals => {
            let vec = {
                let mut query = build_proposals_query(filter, order_by)?;

                if let Some(limit) = limit {
                    query = query.limit(map_limit(limit)?);
                }

                if let Some(offset) = offset {
                    query = query.offset(map_offset(offset)?);
                }

                query.load(conn)
            }
            .map_err(|_| HandleError::InternalError("error searching".to_string()))?;

            Ok(SearchResponse::Proposal(vec))
        }
    }
}

fn map_limit(limit: u64) -> Result<i64, HandleError> {
    limit
        .try_into()
        .map_err(|_| HandleError::BadRequest(format!("limit must be less than: {}", i64::MAX)))
}

fn map_offset(offset: u64) -> Result<i64, HandleError> {
    offset
        .try_into()
        .map_err(|_| HandleError::BadRequest(format!("offset must be less than: {}", i64::MAX)))
}

fn search_count(
    SearchCountQuery { table, filter, .. }: SearchCountQuery,
    conn: &DbConnection,
) -> Result<i64, HandleError> {
    match table {
        Table::Challenges => {
            let query = build_challenges_query(filter, Vec::new())?;

            query
                .count()
                .get_result(conn)
                .map_err(|e| HandleError::InternalError(e.to_string()))
        }
        Table::Proposals => {
            let query = build_proposals_query(filter, Vec::new())?;

            query
                .count()
                .get_result(conn)
                .map_err(|_| HandleError::InternalError("error searching".to_string()))
        }
    }
}
