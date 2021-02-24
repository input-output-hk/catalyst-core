use assert_fs::TempDir;
use iapyx::{IapyxLoad, IapyxLoadConfig, Protocol};
use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::scenario::Controller;
use jormungandr_scenario_tests::Context;
use jormungandr_testing_utils::testing::{network_builder::Seed, node::time};
use jortestkit::prelude::*;
use jortestkit::{
    load::{Configuration, Monitor},
    measurement::Status,
};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vitup::scenario::controller::VitController;
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

#[test]
pub fn load_test_public_100_000_votes() {
    let endpoint = "127.0.0.1:8080";
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let no_of_votes = 100_000;
    let no_of_threads = 5;
    let no_of_wallets = 3_000;

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
        vitup_setup(false, no_of_wallets, testing_directory.path().to_path_buf());

    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
    )
    .unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    let config = build_load_config(endpoint, qr_codes_folder, no_of_threads, no_of_votes);
    let iapyx_load = IapyxLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    println!(
        "{:?}",
        nodes
            .get(0)
            .unwrap()
            .explorer()
            .status()
            .unwrap()
            .data
            .unwrap()
            .status
    );

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
pub fn load_test_private_30_000_votes() {
    let endpoint = "127.0.0.1:8080";
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let no_of_votes = 30_000;
    let no_of_threads = 5;
    let no_of_wallets = 1_000;

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
        vitup_setup(true, no_of_wallets, testing_directory.path().to_path_buf());
    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
    )
    .unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    wait_until_folder_contains_all_qrs(no_of_wallets, &qr_codes_folder);

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

pub fn wait_until_folder_contains_all_qrs<P: AsRef<Path>>(qrs_count: usize, folder: P) {
    loop {
        let qrs = std::fs::read_dir(folder.as_ref()).unwrap();
        if qrs.into_iter().count() < qrs_count {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}

pub fn vitup_setup(
    private: bool,
    no_of_wallets: usize,
    mut testing_directory: PathBuf,
) -> (VitController, Controller, ValidVotePlanParameters, String) {
    let jormungandr = prepare_command(PathBuf::from_str("jormungandr").unwrap());
    let jcli = prepare_command(PathBuf::from_str("jcli").unwrap());
    let seed = Seed::generate(rand::rngs::OsRng);
    let generate_documentation = true;
    let log_level = "info".to_string();

    let context = Context::new(
        seed,
        jormungandr,
        jcli,
        Some(testing_directory.clone()),
        generate_documentation,
        ProgressBarMode::Standard,
        log_level,
    );

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();

    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(0)
        .tally_start_epoch(20)
        .tally_end_epoch(21)
        .slot_duration_in_seconds(5)
        .slots_in_epoch_count(10)
        .proposals_count(1)
        .voting_power(8_000)
        .private(private);

    testing_directory.push(quick_setup.title());
    if testing_directory.exists() {
        std::fs::remove_dir_all(&testing_directory).unwrap();
    }

    let fund_name = quick_setup.fund_name();
    let (vit_controller, controller, vit_parameters) = quick_setup.build(context).unwrap();
    (vit_controller, controller, vit_parameters, fund_name)
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
