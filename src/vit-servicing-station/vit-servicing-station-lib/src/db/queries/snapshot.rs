use crate::{
    db::{
        models::snapshot::{Contributor, Snapshot, Voter},
        schema::{contributors, snapshots, voters},
        DbConnection, DbConnectionPool,
    },
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};

pub async fn query_all_snapshots(pool: &DbConnectionPool) -> Result<Vec<Snapshot>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        snapshots::dsl::snapshots
            .order_by(snapshots::dsl::last_updated.asc())
            .load(&db_conn)
            .map_err(|e| HandleError::InternalError(format!("Error retrieving snapshot: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub async fn query_snapshot_by_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Snapshot, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        snapshots::dsl::snapshots
            .filter(snapshots::dsl::tag.eq(tag))
            .first(&db_conn)
            .map_err(|e| HandleError::NotFound(format!("Error loading snapshot: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub fn put_snapshot(snapshot: Snapshot, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    diesel::replace_into(snapshots::table)
        .values(snapshot.values())
        .execute(&db_conn)
        .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}

pub async fn query_voters_by_voting_key_and_snapshot_tag(
    voting_key: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Voter>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        voters::dsl::voters
            .filter(voters::dsl::voting_key.eq(voting_key))
            .filter(voters::dsl::snapshot_tag.eq(tag))
            .load(&db_conn)
            .map_err(|e| HandleError::NotFound(format!("Error loading voters: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing voters: {}", e)))?
}

pub fn batch_put_voters(
    voters: &[<Voter as Insertable<voters::table>>::Values],
    db_conn: &DbConnection,
) -> Result<(), HandleError> {
    diesel::replace_into(voters::table)
        .values(voters)
        .execute(db_conn)
        .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}

pub async fn query_contributors_by_voting_key_and_voter_group_and_snapshot_tag(
    voting_key: String,
    voting_group: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Contributor>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        contributors::dsl::contributors
            .filter(contributors::dsl::voting_key.eq(voting_key))
            .filter(contributors::dsl::voting_group.eq(voting_group))
            .filter(contributors::dsl::snapshot_tag.eq(tag))
            .load(&db_conn)
            .map_err(|e| HandleError::NotFound(format!("Error loading contributions: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub fn batch_put_contributions(
    contributions: &[<Contributor as Insertable<contributors::table>>::Values],
    db_conn: &DbConnection,
) -> Result<(), HandleError> {
    diesel::replace_into(contributors::table)
        .values(contributions)
        .execute(db_conn)
        .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}
