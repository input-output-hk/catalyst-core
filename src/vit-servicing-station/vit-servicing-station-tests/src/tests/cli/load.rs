use crate::common::data::{multivoteplan_snapshot, ArbitrarySnapshotGenerator};
use crate::common::{
    cli::VitCliCommand,
    data::CsvConverter,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};
<<<<<<< HEAD
=======
use chain_impl_mockchain::testing::scenario::template::{ProposalDefBuilder, VotePlanDefBuilder};
use time::{macros::format_description, OffsetDateTime};
>>>>>>> 5846dff (switch from chrono to time-rs)

#[test]
pub fn load_data_test() {
    let temp_dir = TempDir::new().unwrap();
    let db_file = temp_dir.child("db.sqlite");
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();

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

    let challenges = temp_dir.child("challenges.csv");
    csv_converter
        .challenges(
            snapshot.challenges().iter().cloned().take(1).collect(),
            challenges.path(),
        )
        .unwrap();

    let reviews = temp_dir.child("reviews.csv");
    csv_converter
        .advisor_reviews(snapshot.advisor_reviews(), reviews.path())
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
<<<<<<< HEAD
=======
    let mut vote_plan_builder = VotePlanDefBuilder::new("fund_3");
    vote_plan_builder.owner("committe_wallet_name");
    vote_plan_builder.vote_phases(1, 2, 3);

    for _ in 0..10 {
        let mut proposal_builder = ProposalDefBuilder::new(
            chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
        );
        proposal_builder.options(3);
        proposal_builder.action_off_chain();
        vote_plan_builder.with_proposal(&mut proposal_builder);
    }

    let vote_plan = vote_plan_builder.build();
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour]");
    let mut parameters = ValidVotePlanParameters::from_single(vote_plan);
    parameters.set_voting_power_threshold(8_000);
    parameters.set_voting_start(
        OffsetDateTime::parse("2015-09-05 23:56:04 00", format)
            .unwrap()
            .unix_timestamp(),
    );
    parameters.set_voting_tally_start(
        OffsetDateTime::parse("2015-09-05 23:56:04 00", format)
            .unwrap()
            .unix_timestamp(),
    );
    parameters.set_voting_tally_end(
        OffsetDateTime::parse("2015-09-05 23:56:04 00", format)
            .unwrap()
            .unix_timestamp(),
    );
    parameters.set_next_fund_start_time(
        OffsetDateTime::parse("2015-09-12 23:56:04 00", format)
            .unwrap()
            .unix_timestamp(),
    );
    parameters.set_registration_snapshot_time(
        OffsetDateTime::parse("2015-09-03 20:00:00 00", format)
            .unwrap()
            .unix_timestamp(),
    );

    let mut template = ArbitraryValidVotingTemplateGenerator::new();
    let mut generator = ValidVotePlanGenerator::new(parameters);
    let snapshot = generator.build(&mut template);

>>>>>>> 5846dff (switch from chrono to time-rs)
    let mut db_builder = DbBuilder::new();
    db_builder.with_snapshot(&multivoteplan_snapshot());
    db_builder.build(&temp_dir).unwrap();
}
