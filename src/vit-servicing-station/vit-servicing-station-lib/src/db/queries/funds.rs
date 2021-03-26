use crate::db::{
    models::{challenges::Challenge, funds::Fund, voteplans::Voteplan},
    schema::{
        challenges::dsl as challenges_dsl, funds, funds::dsl as fund_dsl,
        voteplans::dsl as voteplans_dsl,
    },
    DbConnection, DbConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_fund_by_id(id: i32, pool: &DbConnectionPool) -> Result<Fund, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let query_results = (
            diesel::QueryDsl::filter(fund_dsl::funds, fund_dsl::id.eq(id))
                .first::<Fund>(&db_conn)
                .map_err(|_e| HandleError::NotFound("Error loading fund".to_string())),
            diesel::QueryDsl::filter(voteplans_dsl::voteplans, voteplans_dsl::fund_id.eq(id))
                .load::<Voteplan>(&db_conn)
                .map_err(|_e| HandleError::NotFound("Error loading voteplans".to_string())),
            diesel::QueryDsl::filter(challenges_dsl::challenges, challenges_dsl::fund_id.eq(id))
                .load::<Challenge>(&db_conn)
                .map_err(|_e| HandleError::NotFound("Error loading challenges".to_string())),
        );
        match query_results {
            (Ok(mut fund), Ok(mut voteplans), Ok(mut challenges)) => {
                fund.chain_vote_plans.append(&mut voteplans);
                fund.challenges.append(&mut challenges);
                Ok(fund)
            }
            // Any other combination is not valid
            _ => Err(HandleError::NotFound(format!(
                "Error loading fund with id {}",
                id
            ))),
        }
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_fund(pool: &DbConnectionPool) -> Result<Fund, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let fund = fund_dsl::funds
            .first::<Fund>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund".to_string()));
        let fund = match fund {
            Ok(mut fund) => diesel::QueryDsl::filter(
                voteplans_dsl::voteplans,
                voteplans_dsl::fund_id.eq(fund.id),
            )
            .load::<Voteplan>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund voteplans".to_string()))
            .map(|mut voteplans| {
                fund.chain_vote_plans.append(&mut voteplans);
                Ok(fund)
            }),
            Err(e) => Err(e),
        }?;
        match fund {
            Ok(mut fund) => diesel::QueryDsl::filter(
                challenges_dsl::challenges,
                challenges_dsl::fund_id.eq(fund.id),
            )
            .load::<Challenge>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund challenges".to_string()))
            .map(|mut challenges| {
                fund.challenges.append(&mut challenges);
                Ok(fund)
            }),
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))??
}

pub async fn query_all_funds(pool: &DbConnectionPool) -> Result<Vec<Fund>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        fund_dsl::funds
            .load::<Fund>(&db_conn)
            .map_err(|_| HandleError::InternalError("Error retrieving funds".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn insert_fund(fund: Fund, db_conn: &DbConnection) -> QueryResult<Fund> {
    diesel::insert_into(funds::table)
        .values(fund.values())
        .execute(db_conn)?;
    // This can be done in a single query if we move to postgres or any DB that supports `get_result`
    // instead of `execute` in the previous insert
    funds::table.order(fund_dsl::id.desc()).first(db_conn)
}
