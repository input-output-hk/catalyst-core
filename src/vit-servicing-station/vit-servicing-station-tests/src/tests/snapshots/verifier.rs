use crate::common::{clients::RestClient, raw_snapshot::RawSnapshot, snapshot::VotingPower};
use snapshot_lib::SnapshotInfo;

pub fn assert_raw_against_full_snapshot(
    snapshot_entry: &SnapshotInfo,
    raw_snapshot: &RawSnapshot,
    rest_client: &RestClient,
) {
    let voting_power = VotingPower::from(snapshot_entry.clone());
    let voter_info = rest_client
        .voter_info(&raw_snapshot.tag, &snapshot_entry.hir.voting_key.to_hex())
        .unwrap();
    assert_eq!(
        vec![voting_power],
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

pub fn assert_is_empty_raw_against_full_snapshot(
    snapshot_entry: &SnapshotInfo,
    raw_snapshot: &RawSnapshot,
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
