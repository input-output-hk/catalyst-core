use crate::{
    types::snapshot::{Delegation, Delegator, SnapshotVersion, Voter, VoterInfo},
    Error, EventDB,
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};

#[async_trait]
pub trait SnapshotQueries: Sync + Send + 'static {
    async fn get_snapshot_versions(&self) -> Result<Vec<SnapshotVersion>, Error>;
    async fn get_voter(&self, event: String, voting_key: String) -> Result<Voter, Error>;
    async fn get_delegator(
        &self,
        event: String,
        stake_public_key: String,
    ) -> Result<Delegator, Error>;
}

#[async_trait]
impl SnapshotQueries for EventDB {
    async fn get_snapshot_versions(&self) -> Result<Vec<SnapshotVersion>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let rows = conn
            .query(
                "SELECT event
                FROM snapshot;",
                &[],
            )
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let mut snapshot_versions = Vec::with_capacity(rows.len() + 1);
        for row in rows {
            let version = SnapshotVersion::Number(
                row.try_get("event")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
            );
            snapshot_versions.push(version);
        }
        snapshot_versions.push(SnapshotVersion::Latest);
        Ok(snapshot_versions)
    }

    async fn get_voter(&self, _event: String, voting_key: String) -> Result<Voter, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let voter = conn.query_one("SELECT voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final, SUM(contribution.value)::BIGINT as delegations_power, COUNT(contribution.value) AS delegations_count
        FROM voter
        INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id
        INNER JOIN contribution ON contribution.snapshot_id = snapshot.row_id
        WHERE voter.voting_key = $1 AND contribution.voting_key = $1
        GROUP BY voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final;", &[&voting_key]).await.map_err(|err| Error::Unknown(err.to_string()))?;

        let voting_group = voter
            .try_get("voting_group")
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let voting_power = voter
            .try_get("voting_power")
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let total_voting_power_per_group: i64 = conn
            .query_one(
                "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
                FROM voter
                WHERE voter.voting_group = $1;",
                &[&voting_group],
            )
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?
            .try_get("total_voting_power")
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let voting_power_saturation = if total_voting_power_per_group as f64 != 0_f64 {
            voting_power as f64 / total_voting_power_per_group as f64
        } else {
            0_f64
        };
        Ok(Voter {
            voter_info: VoterInfo {
                delegations_power: voter
                    .try_get("delegations_power")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
                delegations_count: voter
                    .try_get("delegations_count")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
                voting_power_saturation,
                voting_power,
                voting_group,
            },
            as_at: voter
                .try_get::<&'static str, NaiveDateTime>("as_at")
                .map_err(|err| Error::Unknown(err.to_string()))?
                .and_local_timezone(Utc)
                .unwrap(),
            last_updated: voter
                .try_get::<&'static str, NaiveDateTime>("last_updated")
                .map_err(|err| Error::Unknown(err.to_string()))?
                .and_local_timezone(Utc)
                .unwrap(),
            is_final: voter
                .try_get("final")
                .map_err(|err| Error::Unknown(err.to_string()))?,
        })
    }

    async fn get_delegator(
        &self,
        _event: String,
        stake_public_key: String,
    ) -> Result<Delegator, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let delegator = conn.query_one("SELECT contribution.voting_key, contribution.voting_group, snapshot.as_at, snapshot.last_updated, snapshot.final
        FROM contribution
        INNER JOIN snapshot ON contribution.snapshot_id = snapshot.row_id
        WHERE contribution.stake_public_key = $1
        LIMIT 1;", &[&stake_public_key]).await.map_err(|err| Error::Unknown(err.to_string()))?;

        let delegation_rows = conn.query("SELECT contribution.voting_key, contribution.voting_group, contribution.voting_weight, contribution.value,  snapshot.as_at, snapshot.last_updated, snapshot.final
        FROM contribution
        INNER JOIN snapshot ON contribution.snapshot_id = snapshot.row_id
        WHERE contribution.stake_public_key = $1;", &[&stake_public_key]).await.map_err(|err| Error::Unknown(err.to_string()))?;

        let mut delegations = Vec::new();
        for row in delegation_rows {
            delegations.push(Delegation {
                voting_key: row
                    .try_get("voting_key")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
                group: row
                    .try_get("voting_group")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
                weight: row
                    .try_get("voting_weight")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
                value: row
                    .try_get("value")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
            })
        }

        let total_power: i64 = conn
            .query_one(
                "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
                FROM voter;",
                &[],
            )
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?
            .try_get("total_voting_power")
            .map_err(|err| Error::Unknown(err.to_string()))?;

        Ok(Delegator {
            raw_power: delegations.iter().map(|delegation| delegation.value).sum(),
            as_at: delegator
                .try_get::<&'static str, NaiveDateTime>("as_at")
                .map_err(|err| Error::Unknown(err.to_string()))?
                .and_local_timezone(Utc)
                .unwrap(),
            last_updated: delegator
                .try_get::<&'static str, NaiveDateTime>("last_updated")
                .map_err(|err| Error::Unknown(err.to_string()))?
                .and_local_timezone(Utc)
                .unwrap(),
            is_final: delegator
                .try_get("final")
                .map_err(|err| Error::Unknown(err.to_string()))?,
            delegations,
            total_power,
        })
    }
}
