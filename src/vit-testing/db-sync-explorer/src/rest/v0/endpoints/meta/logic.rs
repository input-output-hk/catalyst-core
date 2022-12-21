use crate::db::Meta;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use diesel::RunQueryDsl;

pub async fn get_meta_info(context: SharedContext) -> Result<Vec<Meta>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    let mut db_conn = pool.get().map_err(HandleError::Connection)?;

    tokio::task::spawn_blocking(move || {
        use crate::db::schema::meta;
        meta::table
            .load(&mut db_conn)
            .map_err(HandleError::Database)
    })
    .await
    .map_err(HandleError::Join)?
}
