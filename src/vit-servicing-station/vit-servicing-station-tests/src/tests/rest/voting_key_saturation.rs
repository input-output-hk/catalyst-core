use crate::common::{
    data::{self, Snapshot},
    startup::{db::DbBuilder, quick_start, server::ServerBootstrapper},
};

use assert_fs::TempDir;
use crate::common::snapshot::SnapshotBuilder;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData,
    challenges::Challenge,
    funds::Fund,
    groups::Group,
    proposals::{ChallengeType, Proposal},
    voteplans::Voteplan,
};
use crate::common::data::ArbitrarySnapshotGenerator;

#[test] // api/v0/snapshot/voter/{tag}/{voting_key}
pub fn get_voting_key_saturation() { // 2 snapshots required: 1 from SnapshotBuilder, the other from ArbitrarySnapshotGenerator
    let temp_dir = TempDir::new().unwrap();
    let (server, _snapshot) = quick_start(&temp_dir).unwrap();

    let snapshot = SnapshotBuilder::default().build();

    println!("snapshot: {:#?}", snapshot);

    let (hash, token) = data::token();

    let client = server.rest_client_with_token(&hash);

    let snapshot_tag = snapshot.clone().tag;

    println!("snapshot tags from data: {:#?}", snapshot_tag);

    let public_key = snapshot.content.snapshot[0].hir.clone().voting_key.to_string();

    println!("public key: {:#?}", public_key);

    let put_snapshot_response = client.put_snapshot_info(&snapshot);

    println!("put snapshot response: {:#?}", put_snapshot_response);

    let snapshot_tags = client.snapshot_tags();

    println!("snapshot tags from server: {:#?}", snapshot_tags);

    let voter_info = client.voter_info(&snapshot_tag, &public_key);

    println!("voter info: {:#?}", voter_info)
}

