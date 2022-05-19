use assert_fs::TempDir;
use valgrind::ValgrindClient;

use crate::data::{challenges_eq, funds_eq, proposals_eq, reviews_eq};
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::parse_challenges;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::parse_proposals;
use vit_servicing_station_tests::common::data::parse_reviews;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn public_vote_multiple_vote_plans() {
    let proposals_path = PathBuf::from_str("./resources/example/proposals.json").unwrap();
    let challenges_path = PathBuf::from_str("./resources/example/challenges.json").unwrap();
    let funds_path = PathBuf::from_str("./resources/example/funds.json").unwrap();
    let reviews_path = PathBuf::from_str("./resources/example/review.json").unwrap();

    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        proposals_path.clone(),
        challenges_path.clone(),
        funds_path.clone(),
        reviews_path.clone(),
    )
    .unwrap();

    let expected_proposals = parse_proposals(proposals_path).unwrap();
    let expected_challenges = parse_challenges(challenges_path).unwrap();
    let expected_funds = parse_funds(funds_path).unwrap();
    let expected_reviews = parse_reviews(reviews_path).unwrap();

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
        .proposals_count(expected_proposals.len() as u32)
        .challenges_count(expected_challenges.len())
        //TODO: implement review_count in template_generator struct
        .reviews_count(3)
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(false)
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

    let actual_fund = backend_client.funds().unwrap();
    let actual_challenges = backend_client.challenges().unwrap();
    let actual_proposals = backend_client.proposals().unwrap();

    funds_eq(expected_fund, actual_fund);
    challenges_eq(expected_challenges, actual_challenges);
    proposals_eq(expected_proposals, actual_proposals);
    reviews_eq(expected_reviews, backend_client);
}
