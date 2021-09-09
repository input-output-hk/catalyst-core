use crate::common::{
    load::build_load_config, vitup_setup, VoteTiming,
};
use assert_fs::TempDir;
use iapyx::{IapyxLoad, Protocol};
use jortestkit::measurement::Status;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;
use jormungandr_lib::interfaces::VotePlanStatus;
use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;
use std::io::BufReader;
use chain_impl_mockchain::block::Block;
use chain_ser::deser::Deserialize;
use catalyst_toolbox::recovery::tally::recover_ledger_from_logs;
use chain_core::property::Fragment;
use assert_fs::fixture::PathChild;

#[test]
pub fn persistent_log_contains_all_sent_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";

    let version = "2.0";
    let no_of_threads = 2;
    let batch_size = 100;
    let no_of_wallets = 40_000;
    let vote_timing = VoteTiming::new(0, 1, 2);

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(vote_timing.vote_start)
        .tally_start_epoch(vote_timing.tally_start)
        .tally_end_epoch(vote_timing.tally_end)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(60)
        .proposals_count(300)
        .voting_power(31_000)
        .private(false);

    let setup_parameters = quick_setup.parameters().clone();
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

    let config = build_load_config(endpoint, qr_codes_folder, no_of_threads, batch_size, setup_parameters);
    let iapyx_load = IapyxLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    vote_timing.wait_for_tally_start(nodes.get(0).unwrap().rest());

    let mut committee = controller.wallet("committee").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(0).unwrap())
        .unwrap();

    let offline_tally_path = testing_directory.child("offline_tally.json");

    let replay = Replay {
            block0_path: controller.block0_file().to_path_buf(),
            block0_url: None,
            logs_path: controller.working_directory().path().join("persistent_log"),
            output: offline_tally_path.path(),
            output_format: OutputFormat::Json,
            verbose: false
    };
    replay.exec().unwrap();
    let offline_voteplan_status: Vec<VotePlanStatus> =
        serde_json::from_str(jortestkit::file::read_file(offline_tally.path())).unwrap();
  
    let live_voteplan_status = wallet_proxy.vote_plan_status().unwrap();

    vit_station.shutdown();
    wallet_proxy.shutdown();
    
    for node in nodes {
        node.logger()
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
        node.shutdown().unwrap();
    }
    
    controller.finalize();

    assert_eq!(live_voteplan_status,offline_voteplan_status);
}