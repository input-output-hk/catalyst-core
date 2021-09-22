use assert_fs::TempDir;
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::Protocol;
use valgrind::ValgrindClient;

use crate::data::vitup_setup;
use jortestkit::prelude::read_file;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vitup::scenario::network::setup_network;
use vitup::setup::start::QuickVitBackendSettingsBuilder;

#[test]
pub fn private_vote_multiple_vote_plans() {
    let funds_path = PathBuf::from_str("../resources/tests/example/funds.json").unwrap();
    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        PathBuf::from_str("../resources/tests/example/proposals.json").unwrap(),
        PathBuf::from_str("../resources/tests/example/challenges.json").unwrap(),
        funds_path.clone(),
        PathBuf::from_str("../resources/tests/example/review.json").unwrap(),
    )
    .unwrap();
    let expected_funds = parse_funds(funds_path).unwrap();

    if expected_funds.len() > 1 {
        panic!("more than 1 expected fund is not supported");
    }

    let expected_fund = expected_funds.iter().next().unwrap().clone();

    let endpoint = "127.0.0.1:8080";
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .vote_start_epoch(0)
        .tally_start_epoch(1)
        .tally_end_epoch(2)
        .fund_id(expected_fund.id)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(30)
        .proposals_count(template_generator.proposals_count() as u32)
        .challenges_count(template_generator.challenges_count() as usize)
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(true);

    let title = quick_setup.title().clone();
    let (mut vit_controller, mut controller, vit_parameters, _) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());
    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
        "2.0".to_owned(),
    )
    .unwrap();

    let backend_client = ValgrindClient::new(endpoint.to_string(), Default::default());
    let fund = backend_client.funds().unwrap();

    for status in backend_client.vote_plan_statuses().unwrap() {
        let actual_encryption_key = fund
            .chain_vote_plans
            .iter()
            .find(|vote_plan| vote_plan.chain_voteplan_id == status.id.to_string())
            .map(|vote_plan| vote_plan.chain_vote_encryption_key.to_string())
            .expect("expected voting to be private. No encryption key found");
        let expected_encryption_key = read_file(
            testing_directory
                .path()
                .join(&title)
                .join(status.id.to_string() + &"_committees")
                .join("election_public_key.sk"),
        );
        assert_eq!(
            actual_encryption_key, expected_encryption_key,
            "invalid encryption key"
        );
    }

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.shutdown().unwrap();
    }
    controller.finalize();
}
