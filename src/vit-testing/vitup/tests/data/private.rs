use assert_fs::TempDir;
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::ValgrindClient;

use jortestkit::prelude::read_file;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vitup::config::{ConfigBuilder, VoteBlockchainTime};
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn private_vote_multiple_vote_plans() {
    let funds_path = PathBuf::from_str("./resources/example/funds.json").unwrap();
    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        PathBuf::from_str("./resources/example/proposals.json").unwrap(),
        PathBuf::from_str("./resources/example/challenges.json").unwrap(),
        funds_path.clone(),
        PathBuf::from_str("./resources/example/review.json").unwrap(),
    )
    .unwrap();
    let expected_funds = parse_funds(funds_path).unwrap();

    if expected_funds.len() > 1 {
        panic!("more than 1 expected fund is not supported");
    }

    let expected_fund = expected_funds.iter().next().unwrap().clone();

    let testing_directory = TempDir::new().unwrap().into_persistent();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let config = ConfigBuilder::default()
        .vote_timing(vote_timing.into())
        .fund_id(expected_fund.id)
        .slot_duration_in_seconds(2)
        .proposals_count(template_generator.proposals_count() as u32)
        .challenges_count(template_generator.challenges_count())
        //TODO: implement review_count in template_generator struct
        .reviews_count(3)
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(true)
        .build();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (_nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let backend_client = ValgrindClient::new(wallet_proxy.address(), Default::default()).unwrap();
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
                .join(status.id.to_string() + "_committees")
                .join("election_public_key.pk"),
        )
        .unwrap();
        assert_eq!(
            actual_encryption_key, expected_encryption_key,
            "invalid encryption key"
        );
    }
}
