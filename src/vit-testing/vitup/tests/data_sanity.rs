use vitup::scenario::network::setup_network;
use vitup::setup::start::QuickVitBackendSettingsBuilder;
use std::path::PathBuf;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::parse_proposals;
use vit_servicing_station_tests::common::data::parse_challenges;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use jormungandr_scenario_tests::scenario::Controller;
use vitup::scenario::controller::VitController;
use jormungandr_scenario_tests::{Context, ProgressBarMode};
use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::Seed;
use iapyx::Protocol;
use assert_fs::TempDir;
use std::str::FromStr;

#[test]
pub fn public_vote_multiple_vote_plans() {
    let proposals_path = PathBuf::from_str("../resources/external/proposals.json").unwrap();
    let challenges_path = PathBuf::from_str("../resources/external/challenges.json").unwrap();
    let funds_path = PathBuf::from_str("../resources/external/funds.json").unwrap();
    
    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        proposals_path.clone(),
        challenges_path.clone(),
        funds_path.clone(),
    ).unwrap();


    let proposals = parse_proposals(proposals_path).unwrap();
    let challenges = parse_challenges(challenges_path).unwrap();
    let funds = parse_funds(funds_path).unwrap();
    
    let endpoint = "127.0.0.1:8080";
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .vote_start_epoch(0)
        .tally_start_epoch(1)
        .tally_end_epoch(2)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(30)
        .proposals_count(proposals.len() as u32)
        .voting_power(8_000)
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
  
    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.shutdown().unwrap();
    }
    controller.finalize();
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
