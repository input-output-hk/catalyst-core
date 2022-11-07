use std::{convert::TryInto, time::Duration};

use crate::{
    common::{
        clients::RawRestClient,
        raw_snapshot::{RawSnapshot, RawSnapshotBuilder, RawSnapshotUpdater},
        snapshot::{SnapshotBuilder, VotingPower},
        startup::quick_start,
    },
    tests::snapshots::verifier::assert_raw_against_full_snapshot,
};
use assert_fs::TempDir;
use snapshot_lib::SnapshotInfo;

use super::verifier::assert_is_empty_raw_against_full_snapshot;

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

    let snapshot_infos: Vec<SnapshotInfo> = raw_snapshot.clone().try_into().unwrap();

    for snapshot_info in snapshot_infos.iter() {
        assert_raw_against_full_snapshot(snapshot_info, &raw_snapshot, &rest_client);
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

    let snapshot_infos: Vec<SnapshotInfo> = raw_snapshot.clone().try_into().unwrap();

    for snapshot_info in snapshot_infos.iter() {
        assert_is_empty_raw_against_full_snapshot(
            snapshot_info,
            &raw_snapshot,
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

    let first_snapshot_infos: Vec<SnapshotInfo> = first_raw_snapshot.clone().try_into().unwrap();

    let second_snapshot_infos: Vec<SnapshotInfo> = second_raw_snapshot.clone().try_into().unwrap();

    for snapshot_info in first_snapshot_infos.iter() {
        assert_is_empty_raw_against_full_snapshot(
            snapshot_info,
            &first_raw_snapshot,
            second_raw_snapshot.content.update_timestamp,
            &rest_client,
        );
    }

    for snapshot_info in second_snapshot_infos.iter() {
        assert_raw_against_full_snapshot(snapshot_info, &second_raw_snapshot, &rest_client);
    }

    let third_snapshot = SnapshotBuilder::default()
        .with_timestamp(second_raw_snapshot.content.update_timestamp + 1)
        .with_tag(second_raw_snapshot.tag.clone())
        .build();

    rest_client.put_snapshot_info(&third_snapshot).unwrap();

    for snapshot_info in second_snapshot_infos.iter() {
        assert_is_empty_raw_against_full_snapshot(
            snapshot_info,
            &second_raw_snapshot,
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

    let first_snapshot_infos: Vec<SnapshotInfo> = first_raw_snapshot.clone().try_into().unwrap();

    for snapshot_info in first_snapshot_infos.iter() {
        assert_raw_against_full_snapshot(snapshot_info, &first_raw_snapshot, &rest_client);
        assert_raw_against_full_snapshot(snapshot_info, &second_raw_snapshot, &rest_client);
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

    let snapshot_infos: Vec<SnapshotInfo> = raw_snapshot.clone().try_into().unwrap();

    assert_raw_against_full_snapshot(&snapshot_infos[0].clone(), &raw_snapshot, &rest_client);
}
