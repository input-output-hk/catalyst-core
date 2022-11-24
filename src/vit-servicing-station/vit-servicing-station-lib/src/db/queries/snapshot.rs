use crate::{
    db::{
        models::snapshot::{Contribution, Snapshot, Voter},
        schema::{contributions, snapshots, voters},
        DbConnection, DbConnectionPool,
    },
    q,
    utils::collections::dedup_by_key_keep_last,
    v0::errors::HandleError,
};
use diesel::{
    pg::upsert::excluded, query_dsl::methods::ExecuteDsl, Connection, ExpressionMethods, QueryDsl,
    RunQueryDsl,
};

pub async fn query_all_snapshots(pool: &DbConnectionPool) -> Result<Vec<Snapshot>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            snapshots::dsl::snapshots
                .order_by(snapshots::dsl::last_updated.asc())
                .load(&db_conn)
        )
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
        q!(
            db_conn,
            snapshots::dsl::snapshots
                .filter(snapshots::dsl::tag.eq(tag))
                .first(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading snapshot: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub fn put_snapshot(snapshot: Snapshot, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;

    match db_conn {
        DbConnection::Sqlite(db_conn) => diesel::replace_into(snapshots::table)
            .values(snapshot)
            .execute(&db_conn),
        DbConnection::Postgres(db_conn) => {
            // TODO: Find a better way to do this?
            //
            // This is needed when moving from SQLite to Postgres
            // because it used to be a replace_into call which
            // is actually a delete followed by a insert in SQLite and
            // this triggers the 'ON DELETE CASCADE' of voters and contributions
            // tables.
            db_conn.transaction(|| {
                diesel::delete(snapshots::table)
                    .filter(snapshots::tag.eq(&snapshot.tag))
                    .execute(&db_conn)?;

                diesel::insert_into(snapshots::table)
                    .values(snapshot)
                    .execute(&db_conn)
            })
        }
    }
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}

pub async fn query_voters_by_snapshot_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Voter>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            voters::dsl::voters
                .filter(voters::dsl::snapshot_tag.eq(tag))
                .load(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading voters: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing voters: {}", e)))?
}

pub async fn query_voters_by_voting_key_and_snapshot_tag(
    voting_key: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Voter>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            voters::dsl::voters
                .filter(voters::dsl::voting_key.eq(voting_key))
                .filter(voters::dsl::snapshot_tag.eq(tag))
                .load(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading voters: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing voters: {}", e)))?
}

pub async fn query_total_voting_power_by_voting_group_and_snapshot_tag(
    voting_group: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<i64, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            voters::dsl::voters
                .filter(voters::dsl::voting_group.eq(voting_group))
                .filter(voters::dsl::snapshot_tag.eq(tag))
                .load::<Voter>(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading voters: {}", e)))
        .map(|voters| voters.iter().map(|voter| voter.voting_power).sum())
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing voters: {}", e)))?
}

pub fn batch_put_voters(voters: &[Voter], db_conn: &DbConnection) -> Result<(), HandleError> {
    match db_conn {
        DbConnection::Sqlite(db_conn) => ExecuteDsl::<_, diesel::sqlite::Sqlite>::execute(
            diesel::replace_into(voters::table).values(voters),
            db_conn,
        ),
        DbConnection::Postgres(db_conn) => {
            // Postgres will not allow batch inserting if there are any values that would modify the same row twice.
            // SQLite will allow it then, for Postgres, we keep only the latest conflicting value.
            let unique_voters = dedup_by_key_keep_last(voters.iter(), |v| {
                (&v.voting_key, &v.voting_group, &v.snapshot_tag)
            });

            diesel::insert_into(voters::table)
                .values(unique_voters)
                .on_conflict((
                    voters::voting_key,
                    voters::voting_group,
                    voters::snapshot_tag,
                ))
                .do_update()
                .set((voters::voting_power.eq(excluded(voters::voting_power)),))
                .execute(db_conn)
        }
    }
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}

pub async fn query_contributions_by_voting_key_and_voter_group_and_snapshot_tag(
    voting_key: String,
    voting_group: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Contribution>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            contributions::dsl::contributions
                .filter(contributions::dsl::voting_key.eq(voting_key))
                .filter(contributions::dsl::voting_group.eq(voting_group))
                .filter(contributions::dsl::snapshot_tag.eq(tag))
                .load(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading contributions: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub async fn query_contributions_by_snapshot_tag(
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Contribution>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            contributions::dsl::contributions
                .filter(contributions::dsl::snapshot_tag.eq(tag))
                .load(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading contributions: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub async fn query_contributions_by_stake_public_key_and_snapshot_tag(
    stake_public_key: String,
    tag: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Contribution>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            contributions::dsl::contributions
                .filter(contributions::dsl::stake_public_key.eq(stake_public_key))
                .filter(contributions::dsl::snapshot_tag.eq(tag))
                .load(&db_conn)
        )
        .map_err(|e| HandleError::NotFound(format!("Error loading contributions: {}", e)))
    })
    .await
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?
}

pub fn batch_put_contributions(
    contributions: &[Contribution],
    db_conn: &DbConnection,
) -> Result<(), HandleError> {
    match db_conn {
        DbConnection::Sqlite(db_conn) => ExecuteDsl::<_, diesel::sqlite::Sqlite>::execute(
            diesel::replace_into(contributions::table).values(contributions),
            db_conn,
        ),
        DbConnection::Postgres(db_conn) => {
            // Postgres will not allow batch inserting if there are any values that would modify the same row twice.
            // SQLite will allow it then, for Postgres, we keep only the latest conflicting value.
            let unique_contributions = dedup_by_key_keep_last(contributions.iter(), |c| {
                (
                    &c.stake_public_key,
                    &c.voting_group,
                    &c.voting_key,
                    &c.snapshot_tag,
                )
            });

            diesel::insert_into(contributions::table)
                .values(unique_contributions)
                .on_conflict((
                    contributions::stake_public_key,
                    contributions::voting_group,
                    contributions::voting_key,
                    contributions::snapshot_tag,
                ))
                .do_update()
                .set((
                    contributions::reward_address.eq(excluded(contributions::reward_address)),
                    contributions::value.eq(excluded(contributions::value)),
                ))
                .execute(db_conn)
        }
    }
    .map_err(|e| HandleError::InternalError(format!("Error executing request: {}", e)))?;
    Ok(())
}
