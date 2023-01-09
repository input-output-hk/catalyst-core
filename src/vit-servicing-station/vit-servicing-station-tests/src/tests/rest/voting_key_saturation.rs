use crate::common::snapshot::SnapshotBuilder;
use crate::common::startup::quick_start;
use assert_fs::TempDir;

const DIRECT_VOTING_GROUP_NAME: &str = "direct";
const DREP_VOTING_GROUP_NAME: &str = "dreps";

#[test]
pub fn get_voting_key_saturation() {
    let temp_dir = TempDir::new().unwrap();

    let (server, _snapshot) = quick_start(&temp_dir).unwrap();

    let snapshot = SnapshotBuilder::default().build();

    let client = server.rest_client();

    let snapshot_tag = snapshot.clone().tag;

    client.put_snapshot_info(&snapshot).unwrap();

    let total_direct_voting_power = snapshot
        .content
        .snapshot
        .iter()
        .filter(|x| x.hir.voting_group == DIRECT_VOTING_GROUP_NAME)
        .map(|x| u64::from(x.hir.voting_power))
        .sum::<u64>() as f64;

    let total_drep_voting_power = snapshot
        .content
        .snapshot
        .iter()
        .filter(|x| x.hir.voting_group == DREP_VOTING_GROUP_NAME)
        .map(|x| u64::from(x.hir.voting_power))
        .sum::<u64>() as f64;

    for i in 0..snapshot.content.snapshot.len() {
        let key = snapshot.content.snapshot[i].hir.clone().voting_key.to_hex();

        let voter_info = client.voter_info(&snapshot_tag, &key).unwrap();

        let voting_group = snapshot.content.snapshot[i].clone().hir.voting_group;

        assert!(!voter_info.voter_info.is_empty(), "Voter info is empty");

        let voting_power = u64::from(snapshot.content.snapshot[i].hir.clone().voting_power) as f64;

        let expected_voting_key_saturation;

        if voting_group == DIRECT_VOTING_GROUP_NAME {
            expected_voting_key_saturation = voting_power / total_direct_voting_power;
        } else if voting_group == DREP_VOTING_GROUP_NAME {
            expected_voting_key_saturation = voting_power / total_drep_voting_power;
        } else {
            panic!("Voting group not recognized");
        }

        let voting_key_saturation = client
            .voter_info(&snapshot_tag, &key)
            .unwrap()
            .voter_info
            .first()
            .unwrap()
            .voting_power_saturation.voting_power_saturation;

        assert!(expected_voting_key_saturation == voting_key_saturation);

        //assert_eq!(expected_voting_key_saturation, voting_key_saturation);
    }
}
