use crate::db::model::Progress;
use crate::db::{DbPool, TransactionConfirmationRow};
use crate::rest::v0::errors::HandleError;
use crate::BehindDuration;
use diesel::QueryDsl;
use diesel::{sql_function, sql_query, sql_types::Text, RunQueryDsl};
sql_function! (fn decode(string: Text, format: Text) -> Bytea);

pub async fn hash(
    input_hash: String,
    pool: &DbPool,
) -> Result<Vec<TransactionConfirmationRow>, HandleError> {
    let mut db_conn = pool.get().map_err(HandleError::Connection)?;

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::{
            block::{self, block_no, epoch_no, epoch_slot_no, id, slot_no},
            tx::{self, block_id},
        };
        use diesel::ExpressionMethods;
        use diesel::JoinOnDsl;

        let inner_join = block::table.on(id.eq(block_id));

        let table = tx::table.inner_join(inner_join);

        table
            .select((epoch_no, slot_no, epoch_slot_no, block_no))
            .filter(tx::hash.eq(decode(input_hash, "hex")))
            .load(&mut db_conn)
            .map_err(HandleError::Database)
    })
    .await
    .map_err(HandleError::Join)?
}

/// Running sql from dbsync examples:
/// <https://github.com/input-output-hk/cardano-db-sync/blob/master/doc/interesting-queries.md#sync-progress-of-db-sync>
pub async fn sync_progress(pool: &DbPool) -> Result<Vec<Progress>, HandleError> {
    let mut db_conn = pool.get().map_err(HandleError::Connection)?;

    let query = sql_query("select
            100 * (extract (epoch from (max (time) at time zone 'UTC')) - extract (epoch from (min (time) at time zone 'UTC')))
                / (extract (epoch from (now () at time zone 'UTC')) - extract (epoch from (min (time) at time zone 'UTC')))
                as sync_percentage
            from block");

    tokio::task::spawn_blocking(move || query.load(&mut db_conn).map_err(HandleError::Database))
        .await
        .map_err(HandleError::Join)?
}

/// Running sql from dbsync examples:
/// <https://github.com/input-output-hk/cardano-db-sync/blob/master/doc/interesting-queries.md#sync-progress-of-db-sync>
pub async fn behind(pool: &DbPool) -> Result<Vec<BehindDuration>, HandleError> {
    let mut db_conn = pool.get().map_err(HandleError::Connection)?;

    let query = sql_query("select max (time) as behind_by from block");

    tokio::task::spawn_blocking(move || query.load(&mut db_conn).map_err(HandleError::Database))
        .await
        .map_err(HandleError::Join)?
}
