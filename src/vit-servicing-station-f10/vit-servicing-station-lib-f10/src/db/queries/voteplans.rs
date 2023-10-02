use crate::db::{
    models::voteplans::Voteplan, schema::voteplans, schema::voteplans::dsl as voteplans_dsl,
    DbConnection, DbConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_voteplan_by_id(
    id: i32,
    pool: &DbConnectionPool,
) -> Result<Vec<Voteplan>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(voteplans_dsl::voteplans, voteplans_dsl::fund_id.eq(id))
            .load::<Voteplan>(&db_conn)
            .map_err(|_e| HandleError::NotFound(format!("voteplan with id {}", id)))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn insert_voteplan(voteplan: Voteplan, db_conn: &DbConnection) -> QueryResult<Voteplan> {
    diesel::insert_into(voteplans::table)
        .values(voteplan.values())
        .execute(db_conn)?;

    // This can be done in a single query if we move to postgres or any DB that supports `get_result`
    // instead of `execute` in the previous insert
    voteplans::table.order(voteplans::id.desc()).first(db_conn)
}

pub fn batch_insert_voteplans(
    voteplans_slice: &[Voteplan],
    db_conn: &DbConnection,
) -> QueryResult<Vec<Voteplan>> {
    let len = voteplans_slice.len();

    diesel::insert_into(voteplans::table)
        .values(
            voteplans_slice
                .iter()
                .cloned()
                .map(|voteplan| voteplan.values())
                .collect::<Vec<_>>(),
        )
        .execute(db_conn)?;

    // This can be done in a single query if we move to postgres or any DB that supports `get_result`
    // instead of `execute` in the previous insert
    Ok(voteplans::table
        .order(voteplans::id.desc())
        .limit(len as i64)
        .load(db_conn)?
        .iter()
        .cloned()
        .rev()
        .collect())
}
