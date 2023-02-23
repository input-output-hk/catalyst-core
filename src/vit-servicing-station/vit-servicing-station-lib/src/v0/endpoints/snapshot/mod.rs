mod handlers;
mod routes;

use crate::{
    db::{
        models::{
            self,
            snapshot::{Contribution, Voter},
        },
        queries::snapshot::{
            batch_put_contributions, batch_put_voters, put_snapshot, query_all_snapshots,
            query_contributions_by_stake_public_key_and_snapshot_tag,
            query_contributions_by_voting_key_and_voter_group_and_snapshot_tag,
            query_snapshot_by_tag, query_total_voting_power_by_voting_group_and_snapshot_tag,
            query_voters_by_voting_key_and_snapshot_tag,
        },
    },
    v0::{context::SharedContext, errors::HandleError},
};
pub use handlers::{RawSnapshotInput, SnapshotInfoInput};
use itertools::Itertools;
use jormungandr_lib::interfaces::Value;
pub use routes::{filter, update_filter};
use serde::{Deserialize, Serialize};
use snapshot_lib::{
    voting_group::{RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    Dreps, Fraction, RawSnapshot, Snapshot, SnapshotInfo,
};

pub type Tag = String;
pub type Group = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoterInfo {
    pub voting_group: Group,
    pub voting_power: Value,
    pub delegations_power: u64,
    pub delegations_count: u64,
    pub voting_power_saturation: f64,
}

/// Voter information in the current snapshot
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VotersInfo {
    /// A listing of voter information in the current snapshot
    pub voter_info: Vec<VoterInfo>,
    /// Timestamp for the latest update in voter info in the current snapshot
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
    tag: String,
    voting_key: String,
    context: SharedContext,
) -> Result<VotersInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    let snapshot = query_snapshot_by_tag(tag.clone(), pool).await?;
    let mut voter_info = Vec::new();
    let voters =
        query_voters_by_voting_key_and_snapshot_tag(voting_key.clone(), tag.clone(), pool).await?;

    for voter in voters {
        let contributors = query_contributions_by_voting_key_and_voter_group_and_snapshot_tag(
            voting_key.clone(),
            voter.voting_group.clone(),
            tag.clone(),
            pool,
        )
        .await?;

        let total_voting_power_per_group =
            query_total_voting_power_by_voting_group_and_snapshot_tag(
                voter.voting_group.clone(),
                tag.clone(),
                pool,
            )
            .await? as f64;

        voter_info.push(VoterInfo {
            voting_power: Value::from(voter.voting_power as u64),
            delegations_count: contributors.len() as u64,
            delegations_power: contributors
                .iter()
                .map(|contributor| contributor.value as u64)
                .sum(),
            voting_group: voter.voting_group,
            voting_power_saturation: if total_voting_power_per_group != 0_f64 {
                voter.voting_power as f64 / total_voting_power_per_group
            } else {
                0_f64
            },
        })
    }

    Ok(VotersInfo {
        voter_info,
        last_updated: snapshot.last_updated,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatorInfo {
    pub dreps: Vec<String>,
    pub voting_groups: Vec<String>,
    /// Timestamp for the latest update in voter info in the current snapshot
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

#[tracing::instrument(skip(context))]
pub async fn get_delegator_info(
    tag: String,
    stake_public_key: String,
    context: SharedContext,
) -> Result<DelegatorInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    let snapshot = query_snapshot_by_tag(tag.clone(), pool).await?;

    let contributions =
        query_contributions_by_stake_public_key_and_snapshot_tag(stake_public_key, tag, pool)
            .await?;

    Ok(DelegatorInfo {
        dreps: contributions
            .iter()
            .map(|contribution| contribution.voting_key.clone())
            .unique()
            .collect(),
        voting_groups: contributions
            .iter()
            .map(|contribution| contribution.voting_group.clone())
            .unique()
            .collect(),
        last_updated: snapshot.last_updated,
    })
}

pub async fn get_tags(context: SharedContext) -> Result<Vec<Tag>, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    Ok(query_all_snapshots(pool)
        .await?
        .into_iter()
        .map(|snapshot| snapshot.tag)
        .collect())
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(snapshot, context))]
pub async fn update_from_raw_snapshot(
    tag: String,
    snapshot: RawSnapshot,
    update_timestamp: i64,
    min_stake_threshold: Value,
    voting_power_cap: Fraction,
    direct_voters_group: Option<String>,
    representatives_group: Option<String>,
    dreps: Option<Dreps>,
    context: SharedContext,
) -> Result<(), HandleError> {
    let direct_voter = direct_voters_group.unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.into());
    let representative =
        representatives_group.unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.into());
    let assigner = RepsVotersAssigner::new(direct_voter, representative, dreps.unwrap_or_default());
    let snapshot =
        Snapshot::from_raw_snapshot(snapshot, min_stake_threshold, voting_power_cap, &assigner)
            .map_err(|e| HandleError::InternalError(e.to_string()))?
            .to_full_snapshot_info();

    update_from_snapshot_info(tag, snapshot, update_timestamp, context).await
}

pub fn convert_snapshot_to_contrib(
    tag: String,
    snapshot: impl IntoIterator<Item = SnapshotInfo>,
) -> (Vec<Voter>, Vec<Contribution>) {
    let mut contributions = Vec::new();
    let mut voters = Vec::new();
    for entry in snapshot {
        contributions.extend(entry.contributions.into_iter().map(|contribution| {
            Contribution {
                stake_public_key: contribution.stake_public_key,
                reward_address: contribution.reward_address,
                value: contribution
                    .value
                    .try_into()
                    .expect("value should not exceed i64 limit"),
                voting_key: entry.hir.voting_key.to_hex(),
                voting_group: entry.hir.voting_group.clone(),
                snapshot_tag: tag.clone(),
            }
        }));

        voters.push(Voter {
            voting_key: entry.hir.voting_key.to_hex(),
            voting_group: entry.hir.voting_group.clone(),
            voting_power: Into::<u64>::into(entry.hir.voting_power)
                .try_into()
                .expect("value should not exceed i64 limit"),
            snapshot_tag: tag.clone(),
        });
    }
    (voters, contributions)
}

#[tracing::instrument(skip(snapshot, context))]
pub async fn update_from_snapshot_info(
    tag: String,
    snapshot: impl IntoIterator<Item = SnapshotInfo>,
    update_timestamp: i64,
    context: SharedContext,
) -> Result<(), HandleError> {
    let pool = &context.read().await.db_connection_pool;

    put_snapshot(
        models::snapshot::Snapshot {
            tag: tag.clone(),
            last_updated: update_timestamp,
        },
        pool,
    )?;

    let (voters, contributions) = convert_snapshot_to_contrib(tag, snapshot);

    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;

    batch_put_voters(&voters, &db_conn)?;
    batch_put_contributions(&contributions, &db_conn)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::db::migrations::initialize_db_with_migration;
    use crate::v0::context::test::new_db_test_shared_context;
    use jormungandr_lib::crypto::account::Identifier;
    use snapshot_lib::registration::{Delegations, VotingRegistration};
    use snapshot_lib::{KeyContribution, SnapshotInfo, VoterHIR};
    use tracing::Level;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    #[tokio::test]
    pub async fn test_snapshot() {
        let context = new_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn).unwrap();

        let keys = [
            Identifier::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            Identifier::from_hex(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
            Identifier::from_hex(
                "2222222222222222222222222222222222222222222222222222222222222222",
            )
            .unwrap(),
        ];

        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        const UPDATE_TIME1: i64 = 0;
        const UPDATE_TIME2: i64 = 1;

        let key_0_values = [
            VoterInfo {
                voting_group: GROUP1.to_string(),
                voting_power: Value::from(1),
                delegations_power: 2,
                delegations_count: 2,
                voting_power_saturation: 1_f64 / 3_f64,
            },
            VoterInfo {
                voting_group: GROUP2.to_string(),
                voting_power: Value::from(2),
                delegations_power: 2,
                delegations_count: 2,
                voting_power_saturation: 1_f64,
            },
        ];

        let key_1_values = [VoterInfo {
            voting_group: GROUP1.to_string(),
            voting_power: Value::from(2),
            delegations_power: 2,
            delegations_count: 2,
            voting_power_saturation: 2_f64 / 3_f64,
        }];

        let content_a = std::iter::repeat(keys[0].clone())
            .take(key_0_values.len())
            .zip(key_0_values.iter().cloned())
            .map(
                |(
                    voting_key,
                    VoterInfo {
                        voting_group,
                        voting_power,
                        delegations_power: _,
                        delegations_count: _,
                        voting_power_saturation: _,
                    },
                )| SnapshotInfo {
                    contributions: vec![
                        KeyContribution {
                            reward_address: "address_1".to_string(),
                            stake_public_key: "stake_public_key_1".to_string(),
                            value: 1,
                        },
                        KeyContribution {
                            reward_address: "address_2".to_string(),
                            stake_public_key: "stake_public_key_2".to_string(),
                            value: 1,
                        },
                    ],
                    hir: VoterHIR {
                        voting_key,
                        voting_group,
                        voting_power,
                    },
                },
            )
            .chain(
                std::iter::repeat(keys[1].clone())
                    .take(key_1_values.len())
                    .zip(key_1_values.iter().cloned())
                    .map(
                        |(
                            voting_key,
                            VoterInfo {
                                voting_group,
                                voting_power,
                                delegations_power: _,
                                delegations_count: _,
                                voting_power_saturation: _,
                            },
                        )| SnapshotInfo {
                            contributions: vec![
                                KeyContribution {
                                    reward_address: "address_1".to_string(),
                                    stake_public_key: "stake_public_key_1".to_string(),
                                    value: 1,
                                },
                                KeyContribution {
                                    reward_address: "address_2".to_string(),
                                    stake_public_key: "stake_public_key_2".to_string(),
                                    value: 1,
                                },
                            ],
                            hir: VoterHIR {
                                voting_key,
                                voting_group,
                                voting_power,
                            },
                        },
                    ),
            )
            .collect::<Vec<_>>();

        update_from_snapshot_info(
            TAG1.to_string(),
            content_a.clone(),
            UPDATE_TIME1,
            context.clone(),
        )
        .await
        .unwrap();

        let key_2_values = [VoterInfo {
            voting_group: GROUP1.to_string(),
            voting_power: Value::from(3),
            delegations_power: 0,
            delegations_count: 0,
            voting_power_saturation: 0.5_f64,
        }];

        let content_b = std::iter::repeat(keys[2].clone())
            .take(key_2_values.len())
            .zip(key_2_values.iter().cloned())
            .map(
                |(
                    voting_key,
                    VoterInfo {
                        voting_group,
                        voting_power,
                        delegations_power: _,
                        delegations_count: _,
                        voting_power_saturation: _,
                    },
                )| SnapshotInfo {
                    contributions: vec![],
                    hir: VoterHIR {
                        voting_key,
                        voting_group,
                        voting_power,
                    },
                },
            )
            .collect::<Vec<_>>();

        update_from_snapshot_info(
            TAG2.to_string(),
            [content_a, content_b].concat(),
            UPDATE_TIME2,
            context.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            &key_0_values[..],
            &super::get_voters_info(TAG1.to_string(), keys[0].to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info[..],
        );

        assert_eq!(
            &key_1_values[..],
            &super::get_voters_info(TAG1.to_string(), keys[1].to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info[..],
        );

        assert!(
            &super::get_voters_info(TAG1.to_string(), keys[2].to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info
                .is_empty(),
        );

        assert_eq!(
            &key_2_values[..],
            &super::get_voters_info(TAG2.to_string(), keys[2].to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info[..],
        );

        assert_eq!(
            super::get_delegator_info(
                TAG1.to_string(),
                "stake_public_key_1".to_string(),
                context.clone()
            )
            .await
            .unwrap(),
            DelegatorInfo {
                dreps: vec![
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                    "1111111111111111111111111111111111111111111111111111111111111111".to_string()
                ],
                voting_groups: vec!["group1".to_string(), "group2".to_string()],
                last_updated: UPDATE_TIME1,
            }
        );

        assert_eq!(
            super::get_delegator_info(
                TAG1.to_string(),
                "stake_public_key_2".to_string(),
                context.clone()
            )
            .await
            .unwrap(),
            DelegatorInfo {
                dreps: vec![
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                    "1111111111111111111111111111111111111111111111111111111111111111".to_string()
                ],
                voting_groups: vec!["group1".to_string(), "group2".to_string()],
                last_updated: UPDATE_TIME1,
            }
        );

        assert_eq!(
            super::get_delegator_info(
                TAG2.to_string(),
                "stake_public_key_1".to_string(),
                context.clone()
            )
            .await
            .unwrap(),
            DelegatorInfo {
                dreps: vec![
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                    "1111111111111111111111111111111111111111111111111111111111111111".to_string()
                ],
                voting_groups: vec!["group1".to_string(), "group2".to_string()],
                last_updated: UPDATE_TIME2,
            }
        );

        assert_eq!(
            super::get_delegator_info(TAG2.to_string(), "stake_public_key_2".to_string(), context)
                .await
                .unwrap(),
            DelegatorInfo {
                dreps: vec![
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                    "1111111111111111111111111111111111111111111111111111111111111111".to_string()
                ],
                voting_groups: vec!["group1".to_string(), "group2".to_string()],
                last_updated: UPDATE_TIME2,
            }
        );
    }

    #[tokio::test]
    pub async fn test_snapshot_previous_entries_get_deleted() {
        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        const UPDATE_TIME1: i64 = 0;

        let context = new_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn).unwrap();

        let voting_key = Identifier::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let inputs = [
            SnapshotInfo {
                contributions: vec![],
                hir: VoterHIR {
                    voting_key: voting_key.clone(),
                    voting_group: "GROUP1".into(),
                    voting_power: 1.into(),
                },
            },
            SnapshotInfo {
                contributions: vec![],
                hir: VoterHIR {
                    voting_key: voting_key.clone(),
                    voting_group: "GROUP2".into(),
                    voting_power: 1.into(),
                },
            },
        ];

        update_from_snapshot_info(
            TAG1.to_string(),
            inputs.clone(),
            UPDATE_TIME1,
            context.clone(),
        )
        .await
        .unwrap();
        update_from_snapshot_info(
            TAG2.to_string(),
            inputs.clone(),
            UPDATE_TIME1,
            context.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            super::get_voters_info(TAG1.to_string(), voting_key.to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info,
            inputs
                .iter()
                .cloned()
                .map(|snapshot| VoterInfo {
                    voting_group: snapshot.hir.voting_group,
                    voting_power: snapshot.hir.voting_power,
                    delegations_power: snapshot
                        .contributions
                        .iter()
                        .map(|KeyContribution { value, .. }| value)
                        .sum(),
                    delegations_count: snapshot.contributions.len() as u64,
                    voting_power_saturation: 1_f64,
                })
                .collect::<Vec<_>>()
        );

        super::update_from_snapshot_info(
            TAG1.to_string(),
            inputs[0..1].to_vec(),
            UPDATE_TIME1,
            context.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            super::get_voters_info(TAG1.to_string(), voting_key.to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info,
            inputs[0..1]
                .iter()
                .cloned()
                .map(|snapshot| VoterInfo {
                    voting_group: snapshot.hir.voting_group,
                    voting_power: snapshot.hir.voting_power,
                    delegations_power: snapshot
                        .contributions
                        .iter()
                        .map(|KeyContribution { value, .. }| value)
                        .sum(),
                    delegations_count: snapshot.contributions.len() as u64,
                    voting_power_saturation: 1_f64,
                })
                .collect::<Vec<_>>()
        );

        // asserting that TAG2 is untouched, just in case
        assert_eq!(
            super::get_voters_info(TAG2.to_string(), voting_key.to_hex(), context.clone())
                .await
                .unwrap()
                .voter_info,
            inputs
                .iter()
                .cloned()
                .map(|snapshot| VoterInfo {
                    voting_group: snapshot.hir.voting_group,
                    voting_power: snapshot.hir.voting_power,
                    delegations_power: snapshot
                        .contributions
                        .iter()
                        .map(|KeyContribution { value, .. }| value)
                        .sum(),
                    delegations_count: snapshot.contributions.len() as u64,
                    voting_power_saturation: 1_f64,
                })
                .collect::<Vec<_>>()
        );
    }

    async fn get_voters_info<F>(
        tag: &str,
        voting_key: &str,
        filter: &F,
    ) -> Result<Vec<(u64, u64, u64, String)>, StatusCode>
    where
        F: Filter + 'static,
        F::Extract: Reply + Send,
    {
        let result = warp::test::request()
            .path(format!("/snapshot/voter/{}/{}", tag, voting_key).as_ref())
            .reply(filter)
            .await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            return Err(status);
        }

        let req_body = String::from_utf8(result.body().to_vec()).unwrap();
        let VotersInfo {
            voter_info,
            last_updated: _timestamp,
        } = serde_json::from_str(&req_body).unwrap();

        Ok(voter_info
            .into_iter()
            .map(|v| {
                (
                    u64::from(v.voting_power),
                    v.delegations_count,
                    v.delegations_power,
                    v.voting_group,
                )
            })
            .collect::<Vec<_>>())
    }

    #[tokio::test]
    async fn test_snapshot_get_tags_1() {
        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init()
            .unwrap();

        let keys = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "1111111111111111111111111111111111111111111111111111111111111111",
        ];

        let content_a = serde_json::to_string(&SnapshotInfoInput {
            snapshot: vec![
                SnapshotInfo {
                    contributions: vec![
                        KeyContribution {
                            reward_address: "address_1".to_string(),
                            stake_public_key: "stake_public_key_1".to_string(),
                            value: 2,
                        },
                        KeyContribution {
                            reward_address: "address_2".to_string(),
                            stake_public_key: "stake_public_key_2".to_string(),
                            value: 2,
                        },
                    ],
                    hir: VoterHIR {
                        voting_key: Identifier::from_hex(keys[0]).unwrap(),
                        voting_group: GROUP1.to_string(),
                        voting_power: 1.into(),
                    },
                },
                SnapshotInfo {
                    contributions: vec![KeyContribution {
                        reward_address: "address_3".to_string(),
                        stake_public_key: "stake_public_key_3".to_string(),
                        value: 3,
                    }],
                    hir: VoterHIR {
                        voting_key: Identifier::from_hex(keys[0]).unwrap(),
                        voting_group: GROUP2.to_string(),
                        voting_power: 2.into(),
                    },
                },
            ],
            update_timestamp: 0,
        })
        .unwrap();

        let content_b = serde_json::to_string(&SnapshotInfoInput {
            snapshot: vec![SnapshotInfo {
                contributions: vec![],
                hir: VoterHIR {
                    voting_key: Identifier::from_hex(keys[0]).unwrap(),
                    voting_group: GROUP1.to_string(),
                    voting_power: 2.into(),
                },
            }],
            update_timestamp: 1,
        })
        .unwrap();

        let context = new_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn).unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = filter(snapshot_root.clone(), context.clone());
        let put_filter = snapshot_root.and(update_filter(context));

        assert_eq!(
            warp::test::request()
                .path("/snapshot/snapshot_info/tag_a")
                .method("PUT")
                .body(content_a)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            warp::test::request()
                .path("/snapshot/snapshot_info/tag_b")
                .method("PUT")
                .body(content_b)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            get_voters_info("tag_a", keys[0], &filter).await.unwrap(),
            vec![
                (1u64, 2u64, 4u64, GROUP1.to_string()),
                (2u64, 1u64, 3u64, GROUP2.to_string())
            ]
        );

        assert_eq!(
            get_voters_info("tag_b", keys[0], &filter).await.unwrap(),
            vec![(2u64, 0u64, 0u64, GROUP1.to_string())]
        );

        assert!(get_voters_info("tag_c", keys[0], &filter).await.is_err());

        let result = warp::test::request().path("/snapshot").reply(&filter).await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            todo!();
        }

        let mut tags: Vec<String> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        tags.sort_unstable();

        assert_eq!(tags, vec!["tag_a", "tag_b"]);
    }

    #[tokio::test]
    async fn test_snapshot_get_tags_2() {
        let key = "0000000000000000000000000000000000000000000000000000000000000000";

        let content_a = serde_json::to_string(&RawSnapshotInput {
            snapshot: vec![
                VotingRegistration {
                    stake_public_key:
                        "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"
                            .to_string(),
                    voting_power: 2.into(),
                    reward_address:
                        "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"
                            .to_string(),
                    delegations: Delegations::Legacy(Identifier::from_hex(key).unwrap()),
                    voting_purpose: 0,
                },
                VotingRegistration {
                    stake_public_key:
                        "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                            .to_string(),
                    voting_power: 1.into(),
                    reward_address:
                        "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                            .to_string(),
                    delegations: Delegations::Legacy(Identifier::from_hex(key).unwrap()),
                    voting_purpose: 0,
                },
            ]
            .into(),
            update_timestamp: 0,
            min_stake_threshold: 0.into(),
            voting_power_cap: 100.into(),
            direct_voters_group: None,
            representatives_group: None,
            dreps: Some(
                (vec![Identifier::from_hex(key).unwrap()]
                    .into_iter()
                    .collect::<HashSet<Identifier>>())
                .into(),
            ),
        })
        .unwrap();

        let content_b = serde_json::to_string(&RawSnapshotInput {
            snapshot: vec![
                VotingRegistration {
                    stake_public_key:
                        "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"
                            .to_string(),
                    voting_power: 10.into(),
                    reward_address:
                        "0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"
                            .to_string(),
                    delegations: Delegations::Legacy(Identifier::from_hex(key).unwrap()),
                    voting_purpose: 0,
                },
                VotingRegistration {
                    stake_public_key:
                        "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                            .to_string(),
                    voting_power: 1.into(),
                    reward_address:
                        "0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee"
                            .to_string(),
                    delegations: Delegations::Legacy(Identifier::from_hex(key).unwrap()),
                    voting_purpose: 0,
                },
            ]
            .into(),
            update_timestamp: 0,
            min_stake_threshold: 0.into(),
            voting_power_cap: 100.into(),
            direct_voters_group: None,
            representatives_group: None,
            dreps: None,
        })
        .unwrap();

        let context = new_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn).unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = filter(snapshot_root.clone(), context.clone());
        let put_filter = snapshot_root.and(update_filter(context));

        assert_eq!(
            warp::test::request()
                .path("/snapshot/raw_snapshot/tag_a")
                .method("PUT")
                .body(content_a)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            warp::test::request()
                .path("/snapshot/raw_snapshot/tag_b")
                .method("PUT")
                .body(content_b)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            get_voters_info("tag_a", key, &filter).await.unwrap(),
            vec![(3u64, 2u64, 3u64, "rep".to_string())]
        );

        assert_eq!(
            get_voters_info("tag_b", key, &filter).await.unwrap(),
            vec![(11u64, 2u64, 11u64, "direct".to_string())]
        );

        assert!(get_voters_info("tag_c", key, &filter).await.is_err());

        let result = warp::test::request().path("/snapshot").reply(&filter).await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            todo!();
        }

        let mut tags: Vec<String> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        tags.sort_unstable();

        assert_eq!(tags, vec!["tag_a", "tag_b"]);
    }

    #[tokio::test]
    async fn test_put_raw_snapshot() {
        let content = r#"{"snapshot":[{"stake_public_key":"0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663","voting_power":2,"reward_address":"0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663","delegations":"0x0000000000000000000000000000000000000000000000000000000000000000","voting_purpose":0},{"stake_public_key":"0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee","voting_power":1,"reward_address":"0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee","delegations":"0x0000000000000000000000000000000000000000000000000000000000000000","voting_purpose":0}],"update_timestamp":"1970-01-01T00:00:00Z","min_stake_threshold":0,"voting_power_cap": "NaN","direct_voters_group":null,"representatives_group":null}"#;

        let context = new_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn).unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let put_filter = snapshot_root.and(update_filter(context));

        assert_eq!(
            warp::test::request()
                .path("/snapshot/raw_snapshot/tag_a")
                .method("PUT")
                .body(content)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
}
