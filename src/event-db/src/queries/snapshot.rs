use crate::{
    types::snapshot::{Delegation, Delegator, SnapshotVersion, Voter, VoterInfo},
    Error, EventDB,
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};

#[async_trait]
pub trait SnapshotQueries: Sync + Send + 'static {
    async fn get_snapshot_versions(&self) -> Result<Vec<SnapshotVersion>, Error>;
    async fn get_voter(
        &self,
        version: Option<SnapshotVersion>,
        voting_key: String,
    ) -> Result<Voter, Error>;
    async fn get_delegator(
        &self,
        version: Option<SnapshotVersion>,
        stake_public_key: String,
    ) -> Result<Delegator, Error>;
}

impl EventDB {
    const SNAPSHOT_EVENTS_QUERY: &'static str = "SELECT event FROM snapshot;";
    const VOTER_BY_EVENT_QUERY: &'static str = "SELECT voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final, SUM(contribution.value)::BIGINT as delegations_power, COUNT(contribution.value) AS delegations_count
                                            FROM voter
                                            INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id
                                            INNER JOIN contribution ON contribution.snapshot_id = snapshot.row_id
                                            WHERE voter.voting_key = $1 AND contribution.voting_key = $1 AND snapshot.event = $2
                                            GROUP BY voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final;";
    const VOTER_BY_LAST_EVENT_QUERY: &'static str = "SELECT voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final, SUM(contribution.value)::BIGINT as delegations_power, COUNT(contribution.value) AS delegations_count
                                                FROM voter
                                                INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id
                                                INNER JOIN contribution ON contribution.snapshot_id = snapshot.row_id
                                                WHERE voter.voting_key = $1 AND contribution.voting_key = $1 AND snapshot.last_updated = (SELECT MAX(snapshot.last_updated) as last_updated from snapshot)
                                                GROUP BY voter.voting_key, voter.voting_group, voter.voting_power, snapshot.as_at, snapshot.last_updated, snapshot.final;";

    const TOTAL_BY_EVENT_VOTING_QUERY: &'static str =
        "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
        FROM voter
        INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id
        WHERE voter.voting_group = $1 AND snapshot.event = $2;";

    const TOTAL_BY_LAST_EVENT_VOTING_QUERY: &'static str =
        "SELECT SUM(voter.voting_power)::BIGINT as total_voting_power
        FROM voter
        INNER JOIN snapshot ON voter.snapshot_id = snapshot.row_id AND snapshot.last_updated = (SELECT MAX(snapshot.last_updated) as last_updated from snapshot)
        WHERE voter.voting_group = $1;";
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
            .query(Self::SNAPSHOT_EVENTS_QUERY, &[])
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let mut snapshot_versions = Vec::with_capacity(rows.len() + 1);
        for row in rows {
            let version = SnapshotVersion(
                row.try_get("event")
                    .map_err(|err| Error::Unknown(err.to_string()))?,
            );
            snapshot_versions.push(version);
        }
        Ok(snapshot_versions)
    }

    async fn get_voter(
        &self,
        version: Option<SnapshotVersion>,
        voting_key: String,
    ) -> Result<Voter, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let voter = if let Some(version) = &version {
            conn.query_one(Self::VOTER_BY_EVENT_QUERY, &[&voting_key, &version.0])
                .await
                .map_err(|err| Error::Unknown(err.to_string()))?
        } else {
            conn.query_one(Self::VOTER_BY_LAST_EVENT_QUERY, &[&voting_key])
                .await
                .map_err(|err| Error::Unknown(err.to_string()))
                .unwrap()
        };

        let voting_group = voter
            .try_get("voting_group")
            .map_err(|err| Error::Unknown(err.to_string()))?;
        let voting_power = voter
            .try_get("voting_power")
            .map_err(|err| Error::Unknown(err.to_string()))?;

        let total_voting_power_per_group: i64 = if let Some(version) = &version {
            conn.query_one(
                Self::TOTAL_BY_EVENT_VOTING_QUERY,
                &[&voting_group, &version.0],
            )
            .await
            .map_err(|err| Error::Unknown(err.to_string()))?
            .try_get("total_voting_power")
            .map_err(|err| Error::Unknown(err.to_string()))?
        } else {
            conn.query_one(Self::TOTAL_BY_LAST_EVENT_VOTING_QUERY, &[&voting_group])
                .await
                .map_err(|err| Error::Unknown(err.to_string()))
                .unwrap()
                .try_get("total_voting_power")
                .map_err(|err| Error::Unknown(err.to_string()))?
        };

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
        _version: Option<SnapshotVersion>,
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

/// Need to setup and run a test event db instance
/// To do it you can use `cargo make local-event-db-setup`
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, NaiveTime};

    use super::*;
    use crate::test::test_event_db;

    #[tokio::test]
    async fn get_snapshot_versions_test() {
        let event_db = test_event_db().await;

        let snapshot_versions = event_db.get_snapshot_versions().await.unwrap();

        assert_eq!(
            snapshot_versions,
            vec![SnapshotVersion(1), SnapshotVersion(2), SnapshotVersion(3),]
        )
    }

    #[tokio::test]
    async fn get_voter_test() {
        let event_db = test_event_db().await;

        let voter = event_db
            .get_voter(Some(SnapshotVersion(1)), "voting_key_1".to_string())
            .await
            .unwrap();

        assert_eq!(
            voter,
            Voter {
                voter_info: VoterInfo {
                    voting_power: 250,
                    voting_group: "rep".to_string(),
                    delegations_power: 250,
                    delegations_count: 2,
                    voting_power_saturation: 0.625,
                },
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
            }
        );

        let voter = event_db
            .get_voter(None, "voting_key_1".to_string())
            .await
            .unwrap();

        assert_eq!(
            voter,
            Voter {
                voter_info: VoterInfo {
                    voting_power: 250,
                    voting_group: "rep".to_string(),
                    delegations_power: 250,
                    delegations_count: 2,
                    voting_power_saturation: 0.625,
                },
                as_at: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                last_updated: DateTime::<Utc>::from_utc(
                    NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 3, 31).unwrap(),
                        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
                    ),
                    Utc
                ),
                is_final: true,
            }
        );
    }
}
