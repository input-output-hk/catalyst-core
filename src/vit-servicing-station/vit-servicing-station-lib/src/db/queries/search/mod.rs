mod generic;

use crate::{
    db::{models::funds::Fund, schema::funds::dsl::*, DbConnectionPool},
    v0::errors::HandleError,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};

use self::generic::search_query;

pub async fn search_fund_by_name(
    search: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Fund>, HandleError> {
    info!("searching");
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || search_impl(search, &db_conn))
        .await
        .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

fn search_impl(
    search: String,
    conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<Fund>, HandleError> {
    let result = search_query(funds, fund_name, search)
        .load(conn)
        .map_err(|_| HandleError::InternalError("error searching".to_string()))?;
    Ok(result)
}
