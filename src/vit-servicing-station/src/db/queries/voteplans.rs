use crate::db::{
    models::voteplans::Voteplan, schema::voteplans::dsl as voteplans_dsl, DBConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn query_voteplan_by_id(
    id: i32,
    pool: &DBConnectionPool,
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
