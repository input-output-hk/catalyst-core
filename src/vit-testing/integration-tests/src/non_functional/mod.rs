#[cfg(feature = "load-tests")]
pub mod load;
#[cfg(feature = "soak-tests")]
pub mod soak;

use crate::setup::vitup_setup;
use crate::setup::wait_until_folder_contains_all_qrs;
use assert_fs::TempDir;
use iapyx::{IapyxLoad, IapyxLoadConfig, Protocol};
use jormungandr_testing_utils::testing::node::time;
use jortestkit::{
    load::{Configuration, Monitor},
    measurement::Status,
};
use std::path::PathBuf;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

fn private_vote_test_scenario(
    quick_setup: QuickVitBackendSettingsBuilder,
    endpoint: &str,
    no_of_votes: u32,
    no_of_threads: usize,
) {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let wallet_count = quick_setup.parameters().initials.as_ref().unwrap().count();

    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
    )
    .unwrap();

    println!("before qr_codes_folder");

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    wait_until_folder_contains_all_qrs(wallet_count, &qr_codes_folder);

    let config = build_load_config(endpoint, qr_codes_folder, no_of_threads, no_of_votes);
    let iapyx_load = IapyxLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(
            benchmark.status() == Status::Green,
            "too low efficiency {:?} [{:?}]",
            benchmark.efficiency(),
            benchmark.definition().thresholds()
        );
    }

    time::wait_for_epoch(2, nodes.get(0).unwrap().explorer());

    let mut committee = controller.wallet("committee").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(5).unwrap())
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

fn build_load_config(
    address: &str,
    qr_codes_folder: PathBuf,
    threads_no: usize,
    no_of_votes: u32,
) -> IapyxLoadConfig {
    let config = Configuration::requests_per_thread(
        threads_no,
        no_of_votes,
        100,
        Monitor::Progress(100),
        60,
    );

    IapyxLoadConfig {
        config,
        criterion: Some(100),
        address: address.to_string(),
        wallet_mnemonics_file: None,
        qr_codes_folder: Some(qr_codes_folder),
        secrets_folder: None,
        global_pin: "".to_string(),
        read_pin_from_filename: true,
        use_https_for_post: false,
        debug: true,
    }
}
