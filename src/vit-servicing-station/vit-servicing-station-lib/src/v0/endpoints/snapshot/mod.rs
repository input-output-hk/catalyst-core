mod handlers;
mod routes;

use crate::{
    db::{
        models::{
            self,
            snapshot::{Contributor, Voter},
        },
        queries::snapshot::{
            batch_put_contributions, batch_put_voters, put_snapshot, query_all_snapshots,
            query_contributors_by_voting_key_and_voter_group_and_snapshot_tag,
            query_snapshot_by_tag, query_voters_by_voting_key_and_snapshot_tag,
        },
    },
    v0::{context::SharedContext as SharedContext_, errors::HandleError},
};
use diesel::Insertable;
pub use handlers::{RawSnapshotInput, SnapshotInfoInput};
use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};
pub use routes::{filter, update_filter};
use serde::{Deserialize, Serialize};
use snapshot_lib::{
    voting_group::{RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    Fraction, RawSnapshot, Snapshot, SnapshotInfo,
};

pub type Tag = String;
pub type Group = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoterInfo {
    pub voting_group: Group,
    pub voting_power: Value,
    pub delegations_power: u64,
    pub delegations_count: u64,
}

/// Voter information in the current snapshot
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotersInfo {
    /// A listing of voter information in the current snapshot
    pub voter_info: Vec<VoterInfo>,
    /// Timestamp for the latest update in voter info in the current snapshot
    pub last_updated: i64,
}

#[tracing::instrument(skip(context))]
pub async fn get_voters_info(
    tag: &str,
    id: &Identifier,
    context: SharedContext_,
) -> Result<VotersInfo, HandleError> {
    let pool = &context.read().await.db_connection_pool;

    let snapshot = query_snapshot_by_tag(tag.to_string(), pool).await?;
    let mut voter_info = Vec::new();
    let voters =
        query_voters_by_voting_key_and_snapshot_tag(id.to_hex(), tag.to_string(), pool).await?;
    for voter in voters {
        let contributors = query_contributors_by_voting_key_and_voter_group_and_snapshot_tag(
            id.to_hex(),
            voter.voting_group.clone(),
            tag.to_string(),
            pool,
        )
        .await?;
        voter_info.push(VoterInfo {
            voting_power: Value::from(voter.voting_power as u64),
            delegations_count: contributors.len() as u64,
            delegations_power: contributors
                .iter()
                .map(|contributor| contributor.value as u64)
                .sum(),
            voting_group: voter.voting_group,
        })
    }

    Ok(VotersInfo {
        voter_info,
        last_updated: snapshot.last_updated,
    })
}

pub async fn get_tags(context: SharedContext_) -> Result<Vec<Tag>, HandleError> {
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
    tag: &str,
    snapshot: RawSnapshot,
    update_timestamp: u64,
    min_stake_threshold: Value,
    voting_power_cap: Fraction,
    direct_voters_group: Option<String>,
    representatives_group: Option<String>,
    context: SharedContext_,
) -> Result<(), HandleError> {
    let direct_voter = direct_voters_group.unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.into());
    let representative =
        representatives_group.unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.into());
    let assigner = RepsVotersAssigner::new(direct_voter, representative);
    let snapshot =
        Snapshot::from_raw_snapshot(snapshot, min_stake_threshold, voting_power_cap, &assigner)
            .map_err(|e| HandleError::InternalError(e.to_string()))?
            .to_full_snapshot_info();

    update_from_shanpshot_info(tag, snapshot, update_timestamp, context).await
}

#[tracing::instrument(skip(snapshot, context))]
pub async fn update_from_shanpshot_info(
    tag: &str,
    snapshot: impl IntoIterator<Item = SnapshotInfo>,
    update_timestamp: u64,
    context: SharedContext_,
) -> Result<(), HandleError> {
    let pool = &context.read().await.db_connection_pool;

    put_snapshot(
        models::snapshot::Snapshot {
            tag: tag.to_string(),
            last_updated: update_timestamp
                .try_into()
                .expect("value should not exceed i64 limit"),
        },
        pool,
    )?;

    let mut contributions = Vec::new();
    let mut voters = Vec::new();
    for entry in snapshot.into_iter() {
        contributions.extend(entry.contributions.into_iter().map(|contribution| {
            Contributor {
                stake_public_key: contribution.stake_public_key,
                reward_address: contribution.reward_address,
                value: contribution
                    .value
                    .try_into()
                    .expect("value should not exceed i64 limit"),
                voting_key: entry.hir.voting_key.to_hex(),
                voting_group: entry.hir.voting_group.clone(),
                snapshot_tag: tag.to_string(),
            }
            .values()
        }));

        voters.push(
            Voter {
                voting_key: entry.hir.voting_key.to_hex(),
                voting_group: entry.hir.voting_group.clone(),
                voting_power: Into::<u64>::into(entry.hir.voting_power)
                    .try_into()
                    .expect("value should not exceed i64 limit"),
                snapshot_tag: tag.to_string(),
            }
            .values(),
        );
    }
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    batch_put_voters(&voters, &db_conn)?;
    batch_put_contributions(&contributions, &db_conn)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::migrations::initialize_db_with_migration;
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use jormungandr_lib::crypto::account::Identifier;
    use snapshot_lib::registration::{Delegations, VotingRegistration};
    use snapshot_lib::{KeyContribution, SnapshotInfo, VoterHIR};
    use tracing::Level;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    #[tokio::test]
    pub async fn test_snapshot() {
        let context = new_in_memmory_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn);

        let keys = [
            Identifier::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            Identifier::from_hex(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .unwrap(),
        ];

        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        const UPDATE_TIME1: u64 = 0;
        const UPDATE_TIME2: u64 = 1;

        let key_0_values = [
            VoterInfo {
                voting_group: GROUP1.to_string(),
                voting_power: Value::from(1),
                delegations_power: 0,
                delegations_count: 0,
            },
            VoterInfo {
                voting_group: GROUP2.to_string(),
                voting_power: Value::from(2),
                delegations_power: 0,
                delegations_count: 0,
            },
        ];

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

        update_from_shanpshot_info(TAG1, content_a.clone(), UPDATE_TIME1, context.clone())
            .await
            .unwrap();

        let key_1_values = [VoterInfo {
            voting_group: GROUP1.to_string(),
            voting_power: Value::from(3),
            delegations_power: 0,
            delegations_count: 0,
        }];

        let content_b = std::iter::repeat(keys[1].clone())
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

        update_from_shanpshot_info(
            TAG2,
            [content_a, content_b].concat(),
            UPDATE_TIME2,
            context.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            &key_0_values[..],
            &super::get_voters_info(TAG1, &keys[0], context.clone())
                .await
                .unwrap()
                .voter_info[..],
        );

        assert!(&super::get_voters_info(TAG1, &keys[1], context.clone())
            .await
            .unwrap()
            .voter_info
            .is_empty(),);

        assert_eq!(
            &key_1_values[..],
            &super::get_voters_info(TAG2, &keys[1], context)
                .await
                .unwrap()
                .voter_info[..],
        );
    }

    #[tokio::test]
    pub async fn test_snapshot_previous_entries_get_deleted() {
        const TAG1: &str = "tag1";
        const TAG2: &str = "tag2";

        const UPDATE_TIME1: u64 = 0;

        let context = new_in_memmory_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn);

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

        update_from_shanpshot_info(TAG1, inputs.clone(), UPDATE_TIME1, context.clone())
            .await
            .unwrap();
        update_from_shanpshot_info(TAG2, inputs.clone(), UPDATE_TIME1, context.clone())
            .await
            .unwrap();

        assert_eq!(
            super::get_voters_info(TAG1, &voting_key, context.clone())
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
                    delegations_count: snapshot.contributions.len() as u64
                })
                .collect::<Vec<_>>()
        );

        super::update_from_shanpshot_info(
            TAG1,
            inputs[0..1].to_vec(),
            UPDATE_TIME1,
            context.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            super::get_voters_info(TAG1, &voting_key, context.clone())
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
                    delegations_count: snapshot.contributions.len() as u64
                })
                .collect::<Vec<_>>()
        );

        // asserting that TAG2 is untouched, just in case
        assert_eq!(
            super::get_voters_info(TAG2, &voting_key, context.clone())
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
                    delegations_count: snapshot.contributions.len() as u64
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
            .path(format!("/snapshot/{}/{}", tag, voting_key).as_ref())
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

        let _e = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init();

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

        let context = new_in_memmory_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn);

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
        let _e = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init();

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
        })
        .unwrap();

        let context = new_in_memmory_db_test_shared_context();
        let db_conn = &context.read().await.db_connection_pool.get().unwrap();
        initialize_db_with_migration(db_conn);

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
            vec![(3u64, 2u64, 3u64, "direct".to_string())]
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
}
