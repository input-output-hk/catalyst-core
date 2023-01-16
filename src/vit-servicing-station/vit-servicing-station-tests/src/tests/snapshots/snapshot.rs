use std::time::Duration;

use crate::common::{
    clients::RawRestClient,
    snapshot::{Snapshot, SnapshotBuilder, SnapshotUpdater, VotingPower},
    startup::quick_start,
};
use assert_fs::TempDir;
use vit_servicing_station_lib::v0::endpoints::snapshot::SnapshotInfoInput;

#[test]
pub fn import_new_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();
    //add more contributions to voters
    let snap_updater = SnapshotUpdater::from(snapshot.clone());
    let contributions = snapshot
        .content
        .snapshot
        .first()
        .unwrap()
        .contributions
        .clone();
    let voting_key = &snapshot.content.snapshot.last().unwrap().hir.voting_key;
    let snapshot = snap_updater
        .add_contributions_to_voter(contributions, voting_key)
        .build();

    rest_client.put_snapshot_info(&snapshot).unwrap();

    assert_eq!(
        vec![snapshot.tag.to_string()],
        rest_client.snapshot_tags().unwrap(),
        "expected tags vs tags taken from REST API"
    );

    for (idx, entry) in snapshot.content.snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
        for contribution in entry.contributions.iter() {
            let delegator_info = rest_client
                .delegator_info(&snapshot.tag, &contribution.stake_public_key)
                .unwrap();
            assert!(delegator_info
                .dreps
                .contains(&entry.hir.voting_key.to_hex()),
                "delegator doesnt contain entry idx: {}",
                idx
            );
            assert!(delegator_info
                .voting_groups
                .contains(&entry.hir.voting_group),
                "delegator doesnt contain voting group of entry idx: {}",
                idx);
            assert_eq!(
                delegator_info.last_updated,
                snapshot.content.update_timestamp,
                "wrong timestamp for entry idx: {}",
                idx
            );
        }
    }
}

#[test]
pub fn reimport_with_empty_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();

    rest_client.put_snapshot_info(&snapshot).unwrap();

    let empty_snapshot = Snapshot {
        tag: snapshot.tag.clone(),
        content: SnapshotInfoInput {
            snapshot: Vec::new(),
            update_timestamp: 0,
        },
    };

    rest_client.put_snapshot_info(&empty_snapshot).unwrap();
    for (idx, entry) in snapshot.content.snapshot.iter().enumerate() {
        let voter_info = rest_client
            .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
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
pub fn replace_snapshot_with_tag() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_snapshot = Snapshot::default();

    rest_client.put_snapshot_info(&first_snapshot).unwrap();

    let second_snapshot = SnapshotBuilder::default()
        .with_timestamp(first_snapshot.content.update_timestamp + 1)
        .build();

    rest_client.put_snapshot_info(&second_snapshot).unwrap();
    for (idx, entry) in first_snapshot.content.snapshot.iter().enumerate() {
        let voter_info = rest_client
            .voter_info(&first_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert!(
            voter_info.voter_info.is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
    for (idx, entry) in second_snapshot.content.snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&second_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "expected non-empty data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_snapshots_with_different_tags() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let first_snapshot = Snapshot::default();

    rest_client.put_snapshot_info(&first_snapshot).unwrap();

    let second_snapshot = SnapshotUpdater::from(first_snapshot.clone())
        .with_tag("fund9")
        .build();

    rest_client.put_snapshot_info(&second_snapshot).unwrap();

    for (idx, entry) in first_snapshot.content.snapshot.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        let voter_info = rest_client
            .voter_info(&first_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power.clone()],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            first_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );

        let voter_info = rest_client
            .voter_info(&second_snapshot.tag, &entry.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            second_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn import_malformed_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client: RawRestClient = server.rest_client_with_token(&data.token_hash()).into();

    let snapshot = Snapshot::default();
    let mut content = serde_json::to_string(&snapshot.content).unwrap();
    content.pop();
    assert!(rest_client
        .put_snapshot_info(&snapshot.tag, content)
        .unwrap()
        .status()
        .is_client_error());
}
#[test]
pub fn import_big_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let mut rest_client = server.rest_client_with_token(&data.token_hash());
    rest_client.set_timeout(Duration::new(600, 0));

    let snapshot = SnapshotBuilder::default()
        .with_tag("big")
        .with_entries_count(100_000)
        .with_groups(vec![
            "drep".to_string(),
            "direct".to_string(),
            "drep2".to_string(),
            "drep3".to_string(),
        ])
        .build();

    rest_client.put_snapshot_info(&snapshot).unwrap();
    let entry = snapshot.content.snapshot[0].clone();
    let voting_power = VotingPower::from(entry.clone());
    let voter_info = rest_client
        .voter_info(&snapshot.tag, &entry.hir.voting_key.to_hex())
        .unwrap();
    assert_eq!(
        vec![voting_power],
        voter_info.voter_info,
        "wrong data for entry idx"
    );
    assert_eq!(
        snapshot.content.update_timestamp, voter_info.last_updated,
        "wrong timestamp for entry idx"
    );
}
