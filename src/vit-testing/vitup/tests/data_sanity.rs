use assert_fs::TempDir;
use iapyx::Protocol;
use iapyx::WalletBackend;
use iapyx::{Challenge, Fund, Proposal};
use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::scenario::Controller;
use jormungandr_scenario_tests::Seed;
use jormungandr_scenario_tests::{Context, ProgressBarMode};
use std::collections::LinkedList;
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::parse_challenges;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::parse_proposals;
use vit_servicing_station_tests::common::data::ChallengeTemplate;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::FundTemplate;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vitup::scenario::controller::VitController;
use vitup::scenario::network::setup_network;
use vitup::setup::start::QuickVitBackendSettingsBuilder;

#[test]
pub fn public_vote_multiple_vote_plans() {
    let proposals_path = PathBuf::from_str("../resources/tests/example/proposals.json").unwrap();
    let challenges_path = PathBuf::from_str("../resources/tests/example/challenges.json").unwrap();
    let funds_path = PathBuf::from_str("../resources/tests/example/funds.json").unwrap();

    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        proposals_path.clone(),
        challenges_path.clone(),
        funds_path.clone(),
    )
    .unwrap();

    let expected_proposals = parse_proposals(proposals_path).unwrap();
    let expected_challenges = parse_challenges(challenges_path).unwrap();
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
        .proposals_count(expected_proposals.len() as u32)
        .challenges_count(expected_challenges.len())
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(false);

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

    std::thread::sleep(std::time::Duration::from_secs(10));

    let backend_client = WalletBackend::new(endpoint.to_string(), Default::default());

    let actual_fund = backend_client.funds().unwrap();
    let actual_challenges = backend_client.challenges().unwrap();
    let actual_proposals = backend_client.proposals().unwrap();

    funds_eq(expected_fund, actual_fund);
    challenges_eq(expected_challenges, actual_challenges);
    proposals_eq(expected_proposals, actual_proposals);

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.shutdown().unwrap();
    }
    controller.finalize();
}

pub fn funds_eq(expected: FundTemplate, actual: Fund) {
    assert_eq!(expected.id, actual.id, "fund id");
    assert_eq!(expected.goal, actual.fund_goal, "fund goal");
    assert_eq!(expected.rewards_info, actual.rewards_info, "rewards info");
    assert_eq!(
        expected.threshold.unwrap() as u64 * 1_000_000,
        actual.voting_power_threshold,
        "threshold"
    );
}

pub fn challenges_eq(expected_list: LinkedList<ChallengeTemplate>, actual_list: Vec<Challenge>) {
    if expected_list.len() != actual_list.len() {
        panic!("challenges count invalid");
    }

    for (expected, actual) in expected_list.iter().zip(actual_list.iter()) {
        assert_eq!(expected.id, actual.id.to_string(), "id");
        assert_eq!(
            expected.challenge_type.to_string(),
            actual.challenge_type.to_string(),
            "challenge type"
        );
        assert_eq!(expected.title, actual.title, "title");
        assert_eq!(expected.description, actual.description, "description");
        assert_eq!(
            expected.rewards_total,
            actual.rewards_total.to_string(),
            "rewards total"
        );
        assert_eq!(
            expected.proposers_rewards,
            actual.proposers_rewards.to_string(),
            "proposer rewards"
        );
        assert_eq!(
            expected.challenge_url, actual.challenge_url,
            "challenge url"
        );
        assert_eq!(
            expected.fund_id.as_ref().unwrap().to_string(),
            actual.fund_id.to_string(),
            "fund id"
        );
    }
}

pub fn proposals_eq(expected_list: LinkedList<ProposalTemplate>, actual_list: Vec<Proposal>) {
    if expected_list.len() != actual_list.len() {
        panic!("proposals count invalid");
    }

    for (expected, actual) in expected_list.iter().zip(actual_list.iter()) {
        assert_eq!(
            expected.internal_id,
            actual.internal_id.to_string(),
            "internal id"
        );
        assert_eq!(
            expected.category_name, actual.proposal_category.category_name,
            "category name"
        );
        assert_eq!(
            expected.proposal_title, actual.proposal_title,
            "proposal title"
        );
        assert_eq!(
            expected.proposal_summary, actual.proposal_summary,
            "proposal summary"
        );
        assert_eq!(
            expected.proposal_funds,
            actual.proposal_funds.to_string(),
            "proposal funds"
        );
        assert_eq!(expected.proposal_url, actual.proposal_url, "proposal url");
        assert_eq!(
            expected.proposal_impact_score,
            actual.proposal_impact_score.to_string(),
            "proposal impact score"
        );
        assert_eq!(
            expected.proposer_name, actual.proposer.proposer_name,
            "proposer name"
        );
        assert_eq!(
            expected.proposer_url, actual.proposer.proposer_url,
            "proposer url"
        );
        assert_eq!(
            expected.proposer_relevant_experience, actual.proposer.proposer_relevant_experience,
            "proposer relevant experience"
        );
        assert_eq!(
            expected.chain_vote_options,
            actual.chain_vote_options.as_csv_string(),
            "chain vote options"
        );
        assert_eq!(
            expected.chain_vote_type, actual.chain_voteplan_payload,
            "chain vote type"
        );
        assert_eq!(
            expected.challenge_id.as_ref().unwrap(),
            &actual.challenge_id.to_string(),
            "challenge id"
        );
    }
}

pub fn context(testing_directory: &PathBuf) -> Context {
    let jormungandr = prepare_command(PathBuf::from_str("jormungandr").unwrap());
    let jcli = prepare_command(PathBuf::from_str("jcli").unwrap());
    let seed = Seed::generate(rand::rngs::OsRng);
    let generate_documentation = true;
    let log_level = "info".to_string();

    Context::new(
        seed,
        jormungandr,
        jcli,
        Some(testing_directory.clone()),
        generate_documentation,
        ProgressBarMode::None,
        log_level,
    )
}

pub fn vitup_setup(
    mut quick_setup: QuickVitBackendSettingsBuilder,
    mut testing_directory: PathBuf,
) -> (VitController, Controller, ValidVotePlanParameters, String) {
    let context = context(&testing_directory);

    testing_directory.push(quick_setup.title());
    if testing_directory.exists() {
        std::fs::remove_dir_all(&testing_directory).unwrap();
    }

    let fund_name = quick_setup.fund_name();
    let (vit_controller, controller, vit_parameters, _) = quick_setup.build(context).unwrap();
    (vit_controller, controller, vit_parameters, fund_name)
}
