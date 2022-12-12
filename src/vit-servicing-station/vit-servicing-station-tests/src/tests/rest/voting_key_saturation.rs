use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};

use assert_fs::TempDir;
use itertools::Itertools;
use crate::common::snapshot::SnapshotBuilder;

#[test] // api/v0/snapshot/voter/{tag}/{voting_key}
pub fn get_voting_key_saturation() {
    let temp_dir = TempDir::new().unwrap();

    let snapshot = SnapshotBuilder::default().with_tag("tag").build();

    let (hash, token) = data::token();

    let groups = snapshot
        .content
        .snapshot
        .iter()
        .map(|x| x.hir.voting_group.clone());
        //.unique();
        //.collect();

    println!("group map: {:#?}", groups);

    let db_path = DbBuilder::new()
        .with_token(token)
        //.with_groups(groups)
        .build(&temp_dir).unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_path)
        .start(&temp_dir).unwrap();

    let client = server.rest_client_with_token(&hash);

    //println!("snapshot content: {:#?}", snapshot.content);

    let public_key = snapshot.content.snapshot[0].contributions[0].clone().stake_public_key;

    println!("public key: {:#?}", public_key);

    let put_snapshot_response = client.put_snapshot_info(&snapshot);

    println!("put snapshot response: {:#?}", put_snapshot_response);

    let snapshot_tags = client.snapshot_tags();

    println!("snapshot tags: {:#?}", snapshot_tags);

    let voter_info = client.voter_info("tag", &public_key);

    println!("voter info: {:#?}", voter_info)
}