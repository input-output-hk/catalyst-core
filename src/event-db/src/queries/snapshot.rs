use crate::{
    types::snapshot::{Delegator, Voter},
    Error, EventDB,
};
use async_trait::async_trait;

#[async_trait]
pub trait SnapshotQueries: Sync + Send + 'static {
    async fn get_snapshot_versions(&self) -> Result<Vec<i32>, Error>;
    async fn get_voter(&self, event: String, voting_key: String) -> Result<Voter, Error>;
    async fn get_delegator(
        &self,
        event: String,
        stake_public_key: String,
    ) -> Result<Delegator, Error>;
}

#[async_trait]
impl SnapshotQueries for EventDB {
    async fn get_snapshot_versions(&self) -> Result<Vec<i32>, Error> {
        let conn = self.pool.get().await?;
        let rows = conn.query("SELECT event from snapshot;", &[]).await?;
        let mut snapshot_versions = Vec::new();
        for row in rows {
            let version = row.try_get("event")?;
            snapshot_versions.push(version);
        }
        Ok(snapshot_versions)
    }

    async fn get_voter(&self, _event: String, _voting_key: String) -> Result<Voter, Error> {
        let conn = self.pool.get().await?;
        let _row = conn.query_one("SELECT ", &[]).await?;

        Ok(Default::default())
    }

    async fn get_delegator(
        &self,
        _event: String,
        _stake_public_key: String,
    ) -> Result<Delegator, Error> {
        let conn = self.pool.get().await?;
        let _row = conn.query_one("SELECT ", &[]).await?;

        Ok(Default::default())
    }
}
