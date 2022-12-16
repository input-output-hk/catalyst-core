use crate::common::{
    data::{self},
    startup::{quick_start},
};
use assert_fs::TempDir;
use crate::common::snapshot::SnapshotBuilder;

#[test] // api/v0/snapshot/voter/{tag}/{voting_key}
pub fn get_voting_key_saturation() { // 2 snapshots required: 1 from SnapshotBuilder, the other from ArbitrarySnapshotGenerator
    let temp_dir = TempDir::new().unwrap();
    let (server, _snapshot) = quick_start(&temp_dir).unwrap();

    let snapshot = SnapshotBuilder::default().build();

    let (hash, _token) = data::token();

    let client = server.rest_client_with_token(&hash);

    let snapshot_tag = snapshot.clone().tag;

    client.put_snapshot_info(&snapshot).unwrap();

    let total_voting_power = snapshot.content.snapshot.iter().map(|x| u64::from(x.hir.voting_power)).sum::<u64>();

    println!("total voting power before cast: {:#?}", total_voting_power);

    let total_voting_power_f = total_voting_power as f64;

    println!("total voting power after cast: {:#?}", total_voting_power_f);

    for i in 0..snapshot.content.snapshot.len() {
        let key = snapshot.content.snapshot[i].hir.clone().voting_key.to_hex();

        let voter_info = client.voter_info(&snapshot_tag, &key).unwrap();

        assert!(voter_info.voter_info.len() > 0, "Voter info is empty");

        println!("voting power before cast: {:#?}", snapshot.content.snapshot[i].hir.clone().voting_power);

        let voting_power = u64::from(snapshot.content.snapshot[i].hir.clone().voting_power) as f64;

        println!("voting power after cast: {:#?}", voting_power);

        let expected_voting_key_saturation = voting_power / total_voting_power_f;

        println!("expected_voting_key_saturation: {:#?}", expected_voting_key_saturation);

        let voting_key_saturation = client.voter_info(&snapshot_tag, &key).unwrap().voter_info.first().unwrap().voting_power_saturation;

        println!("voting_key_saturation: {:#?}", voting_key_saturation);

        assert_eq!(expected_voting_key_saturation, voting_key_saturation);
    }
}

