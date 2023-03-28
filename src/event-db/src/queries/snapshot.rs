use crate::{
    types::snapshot::{Delegator, Voter, VoterInfo},
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
        let rows = conn
            .query(
                "
        SELECT event 
        FROM snapshot;",
                &[],
            )
            .await?;
        let mut snapshot_versions = Vec::new();
        for row in rows {
            let version = row.try_get("event")?;
            snapshot_versions.push(version);
        }
        Ok(snapshot_versions)
    }

    async fn get_voter(&self, _event: String, voting_key: String) -> Result<Voter, Error> {
        let conn = self.pool.get().await?;
        let row = conn.query_one("
         SELECT voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final
         FROM voter 
         INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id 
         WHERE voter.voting_key = $1;
        ", &[&voting_key]).await?;

        Ok(Voter {
            voter_info: VoterInfo {
                voting_power: row.try_get("voting_power")?,
                voting_group: row.try_get("voting_group")?,
                ..Default::default()
            },
            as_at: row.try_get("as_at")?,
            last_updated: row.try_get("last_updated")?,
            r#final: row.try_get("final")?,
        })
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
