use crate::db::schema::votes;
use crate::db::DbConnection;
use crate::{
    db::{models::vote::Vote, schema::votes::dsl as vote_dsl, DbConnectionPool},
    v0::errors::HandleError,
};
use crate::{execute_q, q};
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_votes_by_caster_and_voteplan_id(
    caster: String,
    voteplan_id: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Vote>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            diesel::QueryDsl::filter(vote_dsl::votes, vote_dsl::caster.eq(&caster))
                .filter(vote_dsl::voteplan_id.eq(&voteplan_id))
                .load::<Vote>(&db_conn)
        )
        .map_err(|_e| HandleError::NotFound("Error loading vote".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_votes_data(
    votes: &[<Vote as Insertable<votes::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    execute_q!(db_conn, diesel::insert_into(votes::table).values(votes))
}
