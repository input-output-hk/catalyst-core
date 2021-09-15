mod private;
mod public;

use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::scenario::Controller;
use jormungandr_scenario_tests::Seed;
use jormungandr_scenario_tests::{Context, ProgressBarMode};
use std::collections::LinkedList;
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::{Challenge, Fund, Proposal, ValgrindClient};

use vit_servicing_station_tests::common::data::ChallengeTemplate;
use vit_servicing_station_tests::common::data::FundTemplate;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ReviewTemplate;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vitup::scenario::controller::VitController;
use vitup::setup::start::QuickVitBackendSettingsBuilder;

pub fn funds_eq(expected: FundTemplate, actual: Fund) {
    assert_eq!(expected.id, actual.id, "fund id");
    assert_eq!(expected.goal, actual.fund_goal, "fund goal");
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

pub fn reviews_eq(expected_list: LinkedList<ReviewTemplate>, backend_client: ValgrindClient) {
    for expected in expected_list.iter() {
        for actuals in backend_client
            .review(&expected.proposal_id)
            .unwrap()
            .values()
        {
            for actual in actuals {
                assert_eq!(
                    expected.proposal_id.to_string(),
                    actual.proposal_id.to_string(),
                    "proposal id"
                );
                assert_eq!(expected.rating_given, actual.rating_given, "rating given");
                assert_eq!(expected.assessor, actual.assessor, "rating_given");
                assert_eq!(expected.note, actual.note.to_string(), "note");
                assert_eq!(expected.tag.to_string(), actual.tag.to_string(), "tag");
            }
        }
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
            expected.chain_vote_options.as_csv_string(),
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
