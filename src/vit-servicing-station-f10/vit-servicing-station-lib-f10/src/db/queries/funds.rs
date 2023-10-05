use crate::db::{
    models::{
        challenges::Challenge,
        funds::{Fund, FundStageDates},
        goals::Goal,
        voteplans::Voteplan,
    },
    schema::{
        challenges::dsl as challenges_dsl, funds, funds::dsl as fund_dsl, goals::dsl as goals_dsl,
        voteplans::dsl as voteplans_dsl,
    },
    DbConnection, DbConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
};
use serde::{Deserialize, Serialize};

fn join_fund(
    mut fund: Fund,
    db_conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Fund, HandleError> {
    let id = fund.id;

    fund.chain_vote_plans = voteplans_dsl::voteplans
        .filter(voteplans_dsl::fund_id.eq(id))
        .load::<Voteplan>(db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading voteplans".to_string()))?;

    fund.challenges = challenges_dsl::challenges
        .filter(challenges_dsl::fund_id.eq(id))
        .load::<Challenge>(db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading challenges".to_string()))?;

    fund.goals = goals_dsl::goals
        .filter(goals_dsl::fund_id.eq(id))
        .load::<Goal>(db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading goals".to_string()))?;

    Ok(fund)
}

pub async fn query_fund_by_id(id: i32, pool: &DbConnectionPool) -> Result<Fund, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let fund = fund_dsl::funds
            .filter(fund_dsl::id.eq(id))
            .first::<Fund>(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund".to_string()))?;

        join_fund(fund, &db_conn)
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundWithNext {
    #[serde(flatten)]
    pub fund: Fund,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<FundNextInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundNextInfo {
    pub id: i32,
    pub fund_name: String,
    #[serde(flatten)]
    pub stage_dates: FundStageDates,
}

pub async fn query_current_fund(pool: &DbConnectionPool) -> Result<FundWithNext, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let funds: Vec<Fund> = fund_dsl::funds
            // TODO: Not sure if sorting by the PK is actually necessary
            //
            // this assumes that the next will be the second inserted
            // and that the current is the first.
            .order(fund_dsl::id)
            .limit(2)
            .load(&db_conn)
            .map_err(|_e| HandleError::NotFound("fund".to_string()))?;

        let mut funds = funds.into_iter();
        let current = funds
            .next()
            .ok_or_else(|| HandleError::NotFound("current found not found".to_string()))?;

        let next = funds.next();

        let current = join_fund(current, &db_conn)?;

        Ok(FundWithNext {
            fund: current,
            next: next.map(|f| FundNextInfo {
                id: f.id,
                fund_name: f.fund_name,
                stage_dates: f.stage_dates,
            }),
        })
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_all_funds(pool: &DbConnectionPool) -> Result<Vec<i32>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        fund_dsl::funds
            .select(fund_dsl::id)
            .load::<i32>(&db_conn)
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

pub async fn put_fund(fund: Fund, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    diesel::replace_into(funds::table)
        .values(fund.values())
        .execute(&db_conn)
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?;

    // TODO:
    // replace the voteplan and challenges too?

    Ok(())
}
