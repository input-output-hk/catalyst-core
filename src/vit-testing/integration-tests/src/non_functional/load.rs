use crate::non_functional::build_load_config;
use crate::non_functional::private_vote_test_scenario;
use crate::setup::vitup_setup;
use assert_fs::TempDir;
use iapyx::{IapyxLoad, Protocol};
use jormungandr_testing_utils::testing::node::time;
use jortestkit::measurement::Status;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

#[test]
pub fn load_test_public_100_000_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";
    let version = "2.0";

    let no_of_votes = 10_000;
    let no_of_threads = 10;
    let no_of_wallets = 40_000;
    let vote_timing = [0, 100, 102];

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(vote_timing[0])
        .tally_start_epoch(vote_timing[1])
        .tally_end_epoch(vote_timing[2])
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(60)
        .proposals_count(1)
        .voting_power(31_000)
        .private(false);

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());

    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
        version.to_owned(),
    )
    .unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    let config = build_load_config(endpoint, qr_codes_folder, no_of_threads, no_of_votes);
    let iapyx_load = IapyxLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    time::wait_for_epoch(10, nodes.get(0).unwrap().explorer());

    let mut committee = controller.wallet("committee").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(0).unwrap())
        .unwrap();

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.logger()
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
        node.shutdown().unwrap();
    }

    controller.finalize();
}

#[test]
pub fn load_test_private_pesimistic() {
    let no_of_votes = 8_000;
    let no_of_threads = 10;
    let no_of_wallets = 4_000;
    let endpoint = "127.0.0.1:8080";

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(0)
        .tally_start_epoch(110)
        .tally_end_epoch(115)
        .slot_duration_in_seconds(20)
        .slots_in_epoch_count(3)
        .proposals_count(250)
        .voting_power(31_000)
        .private(true);

    private_vote_test_scenario(quick_setup, endpoint, no_of_votes, no_of_threads);
}

#[test]
pub fn load_test_private_optimistic() {
    let no_of_votes = 10_000;
    let no_of_threads = 10;
    let no_of_wallets = 20_000;
    let endpoint = "127.0.0.1:8080";

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(6)
        .tally_start_epoch(10)
        .tally_end_epoch(11)
        .slot_duration_in_seconds(20)
        .slots_in_epoch_count(180)
        .proposals_count(500)
        .voting_power(31_000)
        .private(true);

    private_vote_test_scenario(quick_setup, endpoint, no_of_votes, no_of_threads);
}
