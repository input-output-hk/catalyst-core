use crate::db::{Meta, Progress};
use crate::rest::v0::errors::HandleError;
use crate::rest::v0::DataProvider;
use crate::{db, BehindDuration, TransactionConfirmation};
use async_trait::async_trait;
use diesel::RunQueryDsl;

pub struct Provider(pub db::DbPool);

#[async_trait]
impl DataProvider for Provider {
    async fn get_meta_info(&self) -> Result<Vec<Meta>, HandleError> {
        let pool = &self.0;
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

    async fn get_interval_behind_now(&self) -> Result<BehindDuration, HandleError> {
        let pool = &self.0;
        let result = db::query::behind(pool).await?;
        if result.is_empty() || result.len() > 1 {
            Err(HandleError::DatabaseInconsistency(
                "expected only 1 record for maximum block time".to_string(),
            ))
        } else {
            Ok(result[0].clone())
        }
    }

    async fn get_sync_progress(&self) -> Result<Progress, HandleError> {
        let pool = &self.0;
        let result = db::query::sync_progress(pool).await?;

        if result.is_empty() || result.len() > 1 {
            Err(HandleError::DatabaseInconsistency(
                "expected only 1 record for sync progress".to_string(),
            ))
        } else {
            Ok(result[0].clone())
        }
    }

    async fn get_tx_by_hash(
        &self,
        hash: String,
    ) -> Result<Vec<TransactionConfirmation>, HandleError> {
        let pool = &self.0;
        db::query::hash(hash, pool)
            .await
            .map(|x| x.into_iter().map(Into::into).collect())
    }
}
