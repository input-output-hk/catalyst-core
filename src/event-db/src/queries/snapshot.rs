use crate::{
    types::snapshot::{Delegation, Delegator, Voter, VoterInfo},
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
                "SELECT event
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
        let voter = conn.query_one("SELECT voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final, SUM(contribution.value)::BIGINT as delegations_power, COUNT(contribution.value) AS delegations_count
        FROM voter
        INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id
        INNER JOIN contribution ON contribution.snapshot_id = snapshot.row_id
        WHERE voter.voting_key = $1 AND contribution.voting_key = $1
        GROUP BY voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final;", &[&voting_key]).await?;

        let voting_group = voter.try_get("voting_group")?;
        let voting_power = voter.try_get("voting_power")?;

        let total_voting_power_per_group: i64 = conn
            .query_one(
                "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
                FROM voter
                WHERE voter.voting_group = $1;",
                &[&voting_group],
            )
            .await?
            .try_get("total_voting_power")?;

        Ok(Voter {
            voter_info: VoterInfo {
                delegations_power: voter.try_get("delegations_power")?,
                delegations_count: voter.try_get("delegations_count")?,
                voting_power_saturation: if total_voting_power_per_group as f64 != 0_f64 {
                    voting_power as f64 / total_voting_power_per_group as f64
                } else {
                    0_f64
                },
                voting_power,
                voting_group,
            },
            as_at: voter.try_get("as_at")?,
            last_updated: voter.try_get("last_updated")?,
            r#final: voter.try_get("final")?,
        })
    }

    async fn get_delegator(
        &self,
        _event: String,
        stake_public_key: String,
    ) -> Result<Delegator, Error> {
        let conn = self.pool.get().await?;
        let delegator = conn.query_one("SELECT contribution.voting_key, contribution.voting_group, snapshot.as_at, snapshot.last_updated, snapshot.final
        FROM contribution
        INNER JOIN snapshot ON contribution.snapshot_id = snapshot.row_id
        WHERE contribution.stake_public_key = $1
        LIMIT 1;", &[&stake_public_key]).await.unwrap();

        let delegation_rows = conn.query("SELECT contribution.voting_key, contribution.voting_group, contribution.voting_weight, contribution.value,  snapshot.as_at, snapshot.last_updated, snapshot.final
        FROM contribution
        INNER JOIN snapshot ON contribution.snapshot_id = snapshot.row_id
        WHERE contribution.stake_public_key = $1;", &[&stake_public_key]).await.unwrap();

        let mut delegations = Vec::new();
        for row in delegation_rows {
            delegations.push(Delegation {
                voting_key: row.try_get("voting_key")?,
                group: row.try_get("voting_group")?,
                weight: row.try_get("voting_weight")?,
                value: row.try_get("value")?,
            })
        }

        let total_power: i64 = conn
            .query_one(
                "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
                FROM voter;",
                &[],
            )
            .await?
            .try_get("total_voting_power")?;

        Ok(Delegator {
            raw_power: delegations.iter().map(|delegation| delegation.value).sum(),
            as_at: delegator.try_get("as_at")?,
            last_updated: delegator.try_get("last_updated")?,
            r#final: delegator.try_get("final")?,
            delegations,
            total_power,
        })
    }
}
