use crate::common::data::{multivoteplan_snapshot, ArbitrarySnapshotGenerator};
use crate::common::{
    cli::VitCliCommand,
    data::CsvConverter,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};

#[test]
pub fn load_data_test() {
    let temp_dir = TempDir::new().unwrap();
    let db_file = temp_dir.child("db.sqlite");
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();

    let csv_converter = CsvConverter;

    let funds = temp_dir.child("funds.csv");
    csv_converter.funds(snapshot.funds(), funds.path()).unwrap();

    let proposals = temp_dir.child("proposals.csv");
    csv_converter
        .proposals(
            snapshot.proposals().iter().take(1).cloned().collect(),
            proposals.path(),
        )
        .unwrap();

    let voteplans = temp_dir.child("voteplans.csv");
    csv_converter
        .voteplans(
            snapshot.voteplans().iter().take(1).cloned().collect(),
            voteplans.path(),
        )
        .unwrap();

    let challenges = temp_dir.child("challenges.csv");
    csv_converter
        .challenges(
            snapshot.challenges().iter().take(1).cloned().collect(),
            challenges.path(),
        )
        .unwrap();

    let reviews = temp_dir.child("reviews.csv");
    csv_converter
        .advisor_reviews(snapshot.advisor_reviews(), reviews.path())
        .unwrap();

    let goals = temp_dir.child("goals.csv");
    csv_converter
        .goals(
            snapshot.goals().iter().map(From::from).collect(),
            goals.path(),
        )
        .unwrap();

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .db()
        .init()
        .db_url(db_file.path())
        .build()
        .assert()
        .success();

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .csv_data()
        .load()
        .db_url(db_file.path())
        .funds(funds.path())
        .proposals(proposals.path())
        .voteplans(voteplans.path())
        .challenges(challenges.path())
        .advisor_reviews(reviews.path())
        .goals(goals.path())
        .build()
        .assert()
        .success();

    let server = ServerBootstrapper::new()
        .with_db_path(db_file.path().to_str().unwrap())
        .start(&temp_dir)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));
    assert!(server.rest_client().health().is_ok());
}

#[test]
pub fn voting_snapshot_build() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut db_builder = DbBuilder::new();
    db_builder.with_snapshot(&multivoteplan_snapshot());
    db_builder.build(&temp_dir).unwrap();
}
