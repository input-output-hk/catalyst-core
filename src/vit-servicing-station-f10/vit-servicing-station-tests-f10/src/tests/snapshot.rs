use crate::common::{
    clients::RawRestClient,
    snapshot::{Snapshot, SnapshotBuilder, SnapshotUpdater, VotingPower},
    startup::quick_start,
};
use assert_fs::TempDir;

#[test]
pub fn import_new_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();

    rest_client.put_snapshot(&snapshot).unwrap();

    assert_eq!(
        vec![snapshot.tag.to_string()],
        rest_client.snapshot_tags().unwrap(),
        "expected tags vs tags taken from REST API"
    );

    for (idx, entry) in snapshot.content.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        assert_eq!(
            vec![voting_power],
            rest_client
                .voting_power(&snapshot.tag, &entry.voting_key.to_hex())
                .unwrap(),
            "wrong data for entry idx: {}",
            idx
        );
    }
}

#[test]
pub fn reimport_with_empty_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = Snapshot::default();

    rest_client.put_snapshot(&snapshot).unwrap();

    let empty_snapshot = Snapshot {
        tag: snapshot.tag.clone(),
        content: Vec::new(),
    };

    rest_client.put_snapshot(&empty_snapshot).unwrap();
    for (idx, entry) in snapshot.content.iter().enumerate() {
        assert!(
            rest_client
                .voting_power(&snapshot.tag, &entry.voting_key.to_hex())
                .unwrap()
                .is_empty(),
            "expected empty data for entry idx: {}",
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

    rest_client.put_snapshot(&first_snapshot).unwrap();

    let second_snapshot = Snapshot::default();

    rest_client.put_snapshot(&second_snapshot).unwrap();
    for (idx, entry) in first_snapshot.content.iter().enumerate() {
        assert!(
            rest_client
                .voting_power(&first_snapshot.tag, &entry.voting_key.to_hex())
                .unwrap()
                .is_empty(),
            "expected empty data for entry idx: {}",
            idx
        );
    }
    for (idx, entry) in second_snapshot.content.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        assert_eq!(
            vec![voting_power],
            rest_client
                .voting_power(&second_snapshot.tag, &entry.voting_key.to_hex())
                .unwrap(),
            "expected non-empty data for entry idx: {}",
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

    rest_client.put_snapshot(&first_snapshot).unwrap();

    let second_snapshot = SnapshotUpdater::from(first_snapshot.clone())
        .with_tag("fund9")
        .build();

    rest_client.put_snapshot(&second_snapshot).unwrap();

    for (idx, entry) in first_snapshot.content.iter().enumerate() {
        let voting_power = VotingPower::from(entry.clone());
        assert_eq!(
            vec![voting_power.clone()],
            rest_client
                .voting_power(&first_snapshot.tag, &entry.voting_key.to_hex())
                .unwrap(),
            "wrong data for entry idx: {}",
            idx
        );
        assert_eq!(
            vec![voting_power],
            rest_client
                .voting_power(&second_snapshot.tag, &entry.voting_key.to_hex())
                .unwrap(),
            "wrong data for entry idx: {}",
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
        .put_snapshot(&snapshot.tag, content)
        .unwrap()
        .status()
        .is_client_error());
}
#[test]
pub fn import_big_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

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

    rest_client.put_snapshot(&snapshot).unwrap();
    let entry = snapshot.content[0].clone();
    let voting_power = VotingPower::from(entry.clone());
    assert_eq!(
        vec![voting_power],
        rest_client
            .voting_power(&snapshot.tag, &entry.voting_key.to_hex())
            .unwrap(),
        "wrong data for entry idx"
    );
}
