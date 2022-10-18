use std::time::Duration;

use crate::common::{
    clients::{RawRestClient, RestClient},
    raw_snapshot::{RawSnapshot, RawSnapshotBuilder, RawSnapshotExtension, RawSnapshotUpdater},
    snapshot::{SnapshotBuilder, VotingPower},
    startup::quick_start,
};
use assert_fs::TempDir;
use snapshot_lib::SnapshotInfo;

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

    let assigner = raw_snapshot.clone().into();

    let snapshot_infos = raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    for snapshot_info in snapshot_infos.iter() {
        assert_against_snapshot(snapshot_info, raw_snapshot.clone(), &rest_client);
    }
}

#[test]
pub fn reimport_with_empty_raw_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&raw_snapshot).unwrap();

    let empty_snapshot = RawSnapshot::empty(raw_snapshot.tag.clone());

    rest_client.put_raw_snapshot(&empty_snapshot).unwrap();

    let assigner = raw_snapshot.clone().into();

    let snapshot_infos = raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    for snapshot_info in snapshot_infos.iter() {
        assert_is_empty_against_snapshot(
            snapshot_info,
            raw_snapshot.clone(),
            empty_snapshot.content.update_timestamp,
            &rest_client,
        );
    }
}

#[test]
pub fn replace_raw_snapshot_with_same_tag() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_raw_snapshot = RawSnapshot::default();

    rest_client.put_raw_snapshot(&first_raw_snapshot).unwrap();

    let second_raw_snapshot = RawSnapshotBuilder::default()
        .with_timestamp(first_raw_snapshot.content.update_timestamp + 1)
        .build();

    rest_client.put_raw_snapshot(&second_raw_snapshot).unwrap();

    let assigner = first_raw_snapshot.clone().into();

    let first_snapshot_infos = first_raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    let assigner = second_raw_snapshot.clone().into();

    let second_snapshot_infos = second_raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    for snapshot_info in first_snapshot_infos.iter() {
        assert_is_empty_against_snapshot(
            snapshot_info,
            first_raw_snapshot.clone(),
            second_raw_snapshot.content.update_timestamp,
            &rest_client,
        );
    }

    for snapshot_info in second_snapshot_infos.iter() {
        assert_against_snapshot(snapshot_info, second_raw_snapshot.clone(), &rest_client);
    }

    let third_snapshot = SnapshotBuilder::default()
        .with_timestamp(second_raw_snapshot.content.update_timestamp + 1)
        .with_tag(second_raw_snapshot.tag.clone())
        .build();

    rest_client.put_snapshot_info(&third_snapshot).unwrap();

    for snapshot_info in second_snapshot_infos.iter() {
        assert_is_empty_against_snapshot(
            snapshot_info,
            second_raw_snapshot.clone(),
            third_snapshot.content.update_timestamp,
            &rest_client,
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

    let assigner = first_raw_snapshot.clone().into();

    let first_snapshot_infos = first_raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    for snapshot_info in first_snapshot_infos.iter() {
        assert_against_snapshot(snapshot_info, first_raw_snapshot.clone(), &rest_client);
        assert_against_snapshot(snapshot_info, second_raw_snapshot.clone(), &rest_client);
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

    let assigner = raw_snapshot.clone().into();

    let snapshot_infos = raw_snapshot
        .clone()
        .into_full_snapshot_infos(&assigner)
        .unwrap();

    assert_against_snapshot(
        &snapshot_infos[0].clone(),
        raw_snapshot.clone(),
        &rest_client,
    );
}

fn assert_against_snapshot(
    snapshot_entry: &SnapshotInfo,
    raw_snapshot: RawSnapshot,
    rest_client: &RestClient,
) {
    let voting_power = VotingPower::from(snapshot_entry.clone());
    let voter_info = rest_client
        .voter_info(&raw_snapshot.tag, &snapshot_entry.hir.voting_key.to_hex())
        .unwrap();
    assert_eq!(
        vec![voting_power.clone()],
        voter_info.voter_info,
        "wrong data for entry: {:?}",
        snapshot_entry
    );
    assert_eq!(
        raw_snapshot.content.update_timestamp, voter_info.last_updated,
        "wrong timestamp for entry: {:?}",
        snapshot_entry
    );
}

fn assert_is_empty_against_snapshot(
    snapshot_entry: &SnapshotInfo,
    raw_snapshot: RawSnapshot,
    timestamp: i64,
    rest_client: &RestClient,
) {
    let voter_info = rest_client
        .voter_info(&raw_snapshot.tag, &snapshot_entry.hir.voting_key.to_hex())
        .unwrap();
    assert!(
        voter_info.voter_info.is_empty(),
        "expected empty data for entry: {:?}",
        snapshot_entry
    );
    assert_eq!(
        timestamp, voter_info.last_updated,
        "wrong timestamp for entry: {:?}",
        snapshot_entry
    );
}
