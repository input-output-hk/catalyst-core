use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use crate::tests::rest::challenges::data::ArbitrarySnapshotGenerator;

use assert_fs::TempDir;
use pretty_assertions::assert_eq;

#[test]
pub fn challenges_are_sorted_by_insertion_order() {
    let temp_dir = TempDir::new().unwrap();

    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();

    let expected_challenges = snapshot.challenges();

    let db_url = DbBuilder::new().with_snapshot(&snapshot).build().unwrap();

    let server = ServerBootstrapper::new()
        .with_db_path(db_url)
        .start(&temp_dir)
        .unwrap();

    let actual_challenges: Vec<String> = server
        .rest_client_with_token(&snapshot.token_hash())
        .challenges()
        .expect("cannot get challenges")
        .into_iter()
        .map(|x| x.title)
        .collect();

    let expected_challenges: Vec<String> =
        expected_challenges.into_iter().map(|x| x.title).collect();

    assert_eq!(actual_challenges, expected_challenges);
}
