use crate::common::{
    asserts::VotePlanStatusAssert, vitup_setup, wait_until_folder_contains_all_qrs,
};
use assert_fs::TempDir;
use chain_impl_mockchain::key::Hash;
use iapyx::{NodeLoad, NodeLoadConfig};
use jormungandr_lib::interfaces::BlockDate;
use jormungandr_testing_utils::testing::node::time;
use jortestkit::{
    load::{Configuration, Monitor},
    measurement::Status,
};
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VitStartParameters;
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

#[allow(dead_code)]
pub fn private_vote_test_scenario(
    quick_setup: QuickVitBackendSettingsBuilder,
    endpoint: &str,
    no_of_threads: usize,
    batch_size: usize,
) {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let parameters = quick_setup.parameters().clone();
    let wallet_count = parameters.initials.count();
    let vote_tally = parameters.vote_tally;
    let slots_per_epoch = parameters.slots_per_epoch;
    let tally_end = parameters.tally_end;

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
        "2.0".to_owned(),
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("generating qr codes..");

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    wait_until_folder_contains_all_qrs(wallet_count, &qr_codes_folder);

    println!("load test setup..");

    let config = build_load_config(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        batch_size,
        parameters,
    );
    let iapyx_load = NodeLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(
            benchmark.status() == Status::Green,
            "too low efficiency {:?} [{:?}]",
            benchmark.efficiency(),
            benchmark.definition().thresholds()
        );
    }

    let leader_1 = nodes.get(0).unwrap();
    let wallet_node = nodes.get(4).unwrap();
    time::wait_for_epoch(vote_tally as u32, leader_1.rest());

    let mut committee = controller.wallet("committee_1").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    match controller.fragment_sender().send_encrypted_tally(
        &mut committee,
        &vote_plan.clone().into(),
        wallet_node,
    ) {
        Ok(_) => (),
        Err(_) => {
            println!("Encrypted Tally {:?}", leader_1.fragment_logs());
            println!("Encrypted Tally {:?}", wallet_node.fragment_logs());
        }
    };

    let target_date = BlockDate::new(vote_tally as u32, slots_per_epoch / 2);
    time::wait_for_date(target_date.into(), leader_1.rest());

    let active_vote_plans = leader_1.vote_plans().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == Hash::from_str(&vote_plan.id()).unwrap().into())
        .unwrap();

    let shares = controller
        .settings()
        .private_vote_plans
        .get(&fund_name)
        .unwrap()
        .decrypt_tally(&vote_plan_status.clone().into());

    match controller.fragment_sender().send_private_vote_tally(
        &mut committee,
        &vote_plan.clone().into(),
        shares,
        wallet_node,
    ) {
        Ok(_) => (),
        Err(_) => {
            println!("Tally {:?}", leader_1.fragment_logs());
            println!("Tally {:?}", wallet_node.fragment_logs());
        }
    };

    time::wait_for_epoch((tally_end + 10) as u32, leader_1.rest());

    leader_1
        .vote_plans()
        .unwrap()
        .assert_all_proposals_are_tallied();

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.logger()
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
        node.shutdown().unwrap();
    }

    controller.finalize();
}

#[allow(dead_code)]
pub fn build_load_config(
    address: &str,
    qr_codes_folder: PathBuf,
    threads_no: usize,
    batch_size: usize,
    parameters: VitStartParameters,
) -> NodeLoadConfig {
    let config = Configuration::duration(
        threads_no,
        parameters.calculate_vote_duration(),
        100,
        Some(250),
        Monitor::Progress(100),
        60,
        1,
    );

    NodeLoadConfig {
        batch_size,
        use_v1: false,
        config,
        criterion: Some(100),
        address: address.to_string(),
        wallet_mnemonics_file: None,
        qr_codes_folder: Some(qr_codes_folder),
        secrets_folder: None,
        global_pin: "".to_string(),
        reuse_accounts_lazy: false,
        reuse_accounts_early: false,
        read_pin_from_filename: true,
        use_https: false,
        debug: false,
    }
}
