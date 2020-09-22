use crate::common::cli::VitCliCommand;
use crate::common::data::{CsvConverter, Generator};
use crate::common::startup::server::ServerBootstrapper;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};

#[test]
pub fn load_data_test() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let db_file = temp_dir.child("db.sqlite");
    let snapshot = Generator::new().snapshot();
    let csv_converter = CsvConverter;

    let funds = temp_dir.child("funds.csv");
    csv_converter
        .funds(
            snapshot.funds().iter().cloned().take(1).collect(),
            funds.path(),
        )
        .unwrap();

    let proposals = temp_dir.child("proposals.csv");
    csv_converter
        .proposals(
            snapshot.proposals().iter().cloned().take(1).collect(),
            proposals.path(),
        )
        .unwrap();

    let voteplans = temp_dir.child("voteplans.csv");
    csv_converter
        .voteplans(
            snapshot.voteplans().iter().cloned().take(1).collect(),
            voteplans.path(),
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
        .build()
        .assert()
        .success();

    let server = ServerBootstrapper::new()
        .with_db_path(db_file.path().to_str().unwrap())
        .start()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));
    assert!(server.rest_client().health().is_ok());
}
