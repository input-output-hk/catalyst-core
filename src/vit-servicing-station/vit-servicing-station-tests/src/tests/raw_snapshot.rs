use std::time::Duration;

use crate::common::{
    clients::RawRestClient,
    raw_snapshot::{RawSnapshot, RawSnapshotBuilder, RawSnapshotUpdater},
    snapshot::{Snapshot as testSnapshot, SnapshotBuilder, VotingPower},
    startup::quick_start,
};
use assert_fs::TempDir;
use snapshot_lib::{voting_group::RepsVotersAssigner, Snapshot};
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;

#[test]
pub fn import_new_raw_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&raw_snapshot).unwrap();

    assert_eq!(
        vec![raw_snapshot.tag.to_string()],
        rest_client.snapshot_tags().unwrap(),
        "expected tags vs tags taken from REST API"
    );

    let assigner = RepsVotersAssigner::new(
        raw_snapshot.content.direct_voters_group.unwrap(),
        raw_snapshot.content.representatives_group.unwrap(),
    );

    let snapshot = Snapshot::from_raw_snapshot(
        raw_snapshot.content.snapshot,
        raw_snapshot.content.min_stake_threshold,
        raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    for (idx, entry) in snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn reimport_with_empty_raw_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&raw_snapshot).unwrap();

    let empty_snapshot = RawSnapshot {
        tag: raw_snapshot.tag.clone(),
        content: RawSnapshotInput {
            snapshot: Vec::new().into(),
            update_timestamp: 0,
            min_stake_threshold: 0.into(),
            voting_power_cap: 0.into(),
            direct_voters_group: None,
            representatives_group: None,
        },
    };

    rest_client.put_raw_snapshot(&empty_snapshot).unwrap();

    let assigner = RepsVotersAssigner::new(
        raw_snapshot.content.direct_voters_group.unwrap(),
        raw_snapshot.content.representatives_group.unwrap(),
    );

    let snapshot = Snapshot::from_raw_snapshot(
        raw_snapshot.content.snapshot,
        raw_snapshot.content.min_stake_threshold,
        raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    for (idx, entry) in snapshot.iter().enumerate() {
        let voter_info = rest_client
            .voter_info(&raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert!(
            voter_info.voter_info.is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            empty_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn replace_raw_snapshot_with_tag() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&first_raw_snapshot).unwrap();

    let second_raw_snapshot = RawSnapshotBuilder::default()
        .with_timestamp(first_raw_snapshot.content.update_timestamp + 1)
        .build();

    rest_client.put_raw_snapshot(&second_raw_snapshot).unwrap();

    let assigner = RepsVotersAssigner::new(
        first_raw_snapshot.content.direct_voters_group.unwrap(),
        first_raw_snapshot.content.representatives_group.unwrap(),
    );

    let first_snapshot = Snapshot::from_raw_snapshot(
        first_raw_snapshot.content.snapshot,
        first_raw_snapshot.content.min_stake_threshold,
        first_raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    let assigner = RepsVotersAssigner::new(
        second_raw_snapshot.content.direct_voters_group.unwrap(),
        second_raw_snapshot.content.representatives_group.unwrap(),
    );

    let second_snapshot = Snapshot::from_raw_snapshot(
        second_raw_snapshot.content.snapshot,
        second_raw_snapshot.content.min_stake_threshold,
        second_raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    for (idx, entry) in first_snapshot.iter().enumerate() {
        let voter_info = rest_client
            .voter_info(&first_raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert!(
            voter_info.voter_info.is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
    for (idx, entry) in second_snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(
                &second_raw_snapshot.tag.clone(),
                &entry.hir.voting_key.to_hex(),
            )
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "expected non-empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }

    let third_snapshot = SnapshotBuilder::default()
        .with_timestamp(second_raw_snapshot.content.update_timestamp + 1)
        .with_tag(second_raw_snapshot.tag.clone())
        .build();

    rest_client.put_snapshot_info(&third_snapshot).unwrap();

    for (idx, entry) in second_snapshot.iter().enumerate() {
        let voter_info = rest_client
            .voter_info(&second_raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert!(
            voter_info.voter_info.is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            third_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }

    for (idx, entry) in third_snapshot.content.snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&third_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "expected non-empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            third_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_raw_snapshots_with_different_tags() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&first_raw_snapshot).unwrap();

    let second_raw_snapshot = RawSnapshotUpdater::from(first_raw_snapshot.clone())
        .with_tag("fund9")
        .build();

    rest_client.put_raw_snapshot(&second_raw_snapshot).unwrap();

    let assigner = RepsVotersAssigner::new(
        first_raw_snapshot.content.direct_voters_group.unwrap(),
        first_raw_snapshot.content.representatives_group.unwrap(),
    );

    let first_snapshot = Snapshot::from_raw_snapshot(
        first_raw_snapshot.content.snapshot,
        first_raw_snapshot.content.min_stake_threshold,
        first_raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    for (idx, entry) in first_snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&first_raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power.clone()],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            first_raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );

        let voter_info = rest_client
            .voter_info(&second_raw_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_malformed_raw_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client: RawRestClient = server.rest_client_with_token(&data.token_hash()).into();

    let raw_snapshot = RawSnapshot::default();
    let mut content = serde_json::to_string(&raw_snapshot.content).unwrap();
    content.pop();
    assert!(rest_client
        .put_raw_snapshot(&raw_snapshot.tag, content)
        .unwrap()
        .status()
        .is_client_error());
}

#[test]
pub fn import_big_raw_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let mut rest_client = server.rest_client_with_token(&data.token_hash());
    rest_client.set_timeout(Duration::new(600, 0));

    let raw_snapshot = RawSnapshotBuilder::default()
        .with_tag("big")
        .with_voting_registrations_count(100_000)
        .build();

    rest_client.put_raw_snapshot(&raw_snapshot).unwrap();

    let assigner = RepsVotersAssigner::new(
        raw_snapshot.content.direct_voters_group.unwrap(),
        raw_snapshot.content.representatives_group.unwrap(),
    );

    let snapshot = Snapshot::from_raw_snapshot(
        raw_snapshot.content.snapshot,
        raw_snapshot.content.min_stake_threshold,
        raw_snapshot.content.voting_power_cap,
        &assigner,
    )
    .unwrap()
    .to_full_snapshot_info();

    let entry = snapshot[0].clone();
    let voting_power = VotingPower::from(entry.clone());
    let voter_info = rest_client
        .voter_info(&raw_snapshot.tag, &entry.hir.voting_key.to_hex())
        .unwrap();
    assert_eq!(
        vec![voting_power],
        voter_info.voter_info,
        "wrong data for entry idx"
    );
    assert_eq!(
        raw_snapshot.content.update_timestamp, voter_info.last_updated,
        "wrong timestamp for entry idx"
    );
}
