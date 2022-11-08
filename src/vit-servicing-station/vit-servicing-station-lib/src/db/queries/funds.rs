use crate::v0::errors::HandleError;
use crate::{
    db::{
        models::{
            challenges::Challenge,
            funds::{Fund, FundStageDates},
            goals::Goal,
            groups::Group,
            voteplans::Voteplan,
        },
        schema::{
            challenges::dsl as challenges_dsl, funds, funds::dsl as fund_dsl,
            goals::dsl as goals_dsl, groups::dsl as groups_dsl, voteplans::dsl as voteplans_dsl,
        },
        DbConnection, DbConnectionPool,
    },
    q,
};
use diesel::pg::upsert::excluded;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};
use serde::{Deserialize, Serialize};

fn join_fund(mut fund: Fund, db_conn: &DbConnection) -> Result<Fund, HandleError> {
    let id = fund.id;

    fund.chain_vote_plans = q!(
        db_conn,
        voteplans_dsl::voteplans
            .filter(voteplans_dsl::fund_id.eq(id))
            .load::<Voteplan>(db_conn)
    )
    .map_err(|_e| HandleError::NotFound("Error loading voteplans".to_string()))?;

    fund.challenges = q!(
        db_conn,
        challenges_dsl::challenges
            .filter(challenges_dsl::fund_id.eq(id))
            .order_by(challenges_dsl::internal_id.asc())
            .load::<Challenge>(db_conn)
    )
    .map_err(|_e| HandleError::NotFound("Error loading challenges".to_string()))?;

    fund.goals = q!(
        db_conn,
        goals_dsl::goals
            .filter(goals_dsl::fund_id.eq(id))
            .load::<Goal>(db_conn)
    )
    .map_err(|_e| HandleError::NotFound("Error loading goals".to_string()))?;

    fund.groups = q!(
        db_conn,
        groups_dsl::groups
            .filter(groups_dsl::fund_id.eq(id))
            .load::<Group>(db_conn)
    )
    .map_err(|_e| HandleError::NotFound("Error loading groups".to_string()))?
    .into_iter()
    .collect();

    Ok(fund)
}

pub async fn query_fund_by_id(id: i32, pool: &DbConnectionPool) -> Result<Fund, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        let db_conn = &db_conn;

        let fund = q!(
            db_conn,
            fund_dsl::funds
                .filter(fund_dsl::id.eq(id))
                .first::<Fund>(db_conn)
        )
        .map_err(|_e| HandleError::NotFound("fund".to_string()))?;

        join_fund(fund, db_conn)
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
        let db_conn = &db_conn;

        let funds: Vec<Fund> = q!(
            db_conn,
            fund_dsl::funds
                // TODO: Not sure if sorting by the PK is actually necessary
                //
                // this assumes that the next will be the second inserted
                // and that the current is the first.
                .order(fund_dsl::id)
                .limit(2)
                .load(db_conn)
        )
        .map_err(|_e| HandleError::NotFound("fund".to_string()))?;

        let mut funds = funds.into_iter();
        let current = funds
            .next()
            .ok_or_else(|| HandleError::NotFound("current found not found".to_string()))?;

        let next = funds.next();

        let current = join_fund(current, db_conn)?;

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
        q!(
            db_conn,
            fund_dsl::funds.select(fund_dsl::id).load::<i32>(&db_conn)
        )
        .map_err(|_| HandleError::InternalError("Error retrieving funds".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn insert_fund(fund: Fund, db_conn: &DbConnection) -> QueryResult<Fund> {
    q!(
        db_conn,
        diesel::insert_into(funds::table)
            .values(fund.values())
            .execute(db_conn)
    )?;
    // This can be done in a single query if we move to postgres or any DB that supports `get_result`
    // instead of `execute` in the previous insert
    q!(
        db_conn,
        funds::table.order(fund_dsl::id.desc()).first(db_conn)
    )
}

pub fn put_fund(fund: Fund, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    match db_conn {
        DbConnection::Sqlite(db_conn) => diesel::replace_into(funds::table)
            .values(fund.values())
            .execute(&db_conn),
        DbConnection::Postgres(db_conn) => diesel::insert_into(funds::table)
            .values(fund.values())
            .on_conflict(funds::id)
            .do_update()
            .set((
                funds::fund_name.eq(excluded(funds::fund_name)),
                funds::fund_goal.eq(excluded(funds::fund_goal)),
                funds::registration_snapshot_time.eq(excluded(funds::registration_snapshot_time)),
                funds::next_registration_snapshot_time
                    .eq(excluded(funds::next_registration_snapshot_time)),
                funds::voting_power_threshold.eq(excluded(funds::voting_power_threshold)),
                funds::fund_start_time.eq(excluded(funds::fund_start_time)),
                funds::fund_end_time.eq(excluded(funds::fund_end_time)),
                funds::next_fund_start_time.eq(excluded(funds::next_fund_start_time)),
                funds::insight_sharing_start.eq(excluded(funds::insight_sharing_start)),
                funds::proposal_submission_start.eq(excluded(funds::proposal_submission_start)),
                funds::refine_proposals_start.eq(excluded(funds::refine_proposals_start)),
                funds::finalize_proposals_start.eq(excluded(funds::finalize_proposals_start)),
                funds::proposal_assessment_start.eq(excluded(funds::proposal_assessment_start)),
                funds::assessment_qa_start.eq(excluded(funds::assessment_qa_start)),
                funds::snapshot_start.eq(excluded(funds::snapshot_start)),
                funds::voting_start.eq(excluded(funds::voting_start)),
                funds::voting_end.eq(excluded(funds::voting_end)),
                funds::tallying_end.eq(excluded(funds::tallying_end)),
                funds::results_url.eq(excluded(funds::results_url)),
                funds::survey_url.eq(excluded(funds::survey_url)),
            ))
            .execute(&db_conn),
    }
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?;

    // TODO:
    // replace the voteplan and challenges too?

    Ok(())
}
