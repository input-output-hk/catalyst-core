use crate::common::wait_until_folder_contains_all_qrs;
use assert_fs::TempDir;
use chain_impl_mockchain::key::Hash;
use hersir::builder::VotePlanSettings;
use iapyx::{NodeLoad, NodeLoadConfig};
use jormungandr_automation::jormungandr::FragmentNode;
use jormungandr_automation::testing::time;
use jormungandr_lib::interfaces::BlockDate;
use jortestkit::{
    load::{ConfigurationBuilder, Monitor},
    measurement::Status,
};
use std::str::FromStr;
use std::{path::PathBuf, time::Duration};
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::convert_to_blockchain_date;
use vitup::config::Config;
use vitup::testing::{spawn_network, vitup_setup};

#[allow(dead_code)]
pub fn private_vote_test_scenario(
    config: Config,
    endpoint: &str,
    no_of_threads: usize,
    batch_size: usize,
) {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let fund_name = config.data.current_fund.fund_info.fund_name.clone();
    let wallet_count = config.initials.block0.count();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (nodes, _vit_station, _wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    println!("generating qr codes..");

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("qr-codes");

    wait_until_folder_contains_all_qrs(wallet_count, &qr_codes_folder);

    println!("load test setup..");

    let load_config = build_load_config(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        1000,
        batch_size,
        config.clone(),
    );
    let iapyx_load = NodeLoad::new(load_config);
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

    let vote_timing = convert_to_blockchain_date(&config);
    vote_timing.wait_for_tally_start(leader_1.rest());

    let mut committee = controller.wallet("committee_1").unwrap();
    let vote_plan = controller.defined_vote_plan(&fund_name).unwrap();

    let target_date = BlockDate::new(vote_timing.tally_end, vote_timing.slots_per_epoch / 2);
    time::wait_for_date(target_date, leader_1.rest());

    let active_vote_plans = leader_1.rest().vote_plan_statuses().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == Hash::from_str(&vote_plan.id()).unwrap().into())
        .unwrap();

    let shares = {
        match controller
            .settings()
            .vote_plans
            .iter()
            .find(|(key, _)| key.alias == fund_name)
            .map(|(_, vote_plan)| vote_plan)
            .unwrap()
        {
            VotePlanSettings::Public(_) => panic!("unexpected public voteplan"),
            VotePlanSettings::Private { keys, vote_plan: _ } => keys
                .decrypt_tally(&vote_plan_status.clone().into())
                .unwrap(),
        }
    };

    let fragment_sender = FragmentSender::from(&controller.settings().block0);

    match fragment_sender.send_private_vote_tally(
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

    time::wait_for_epoch((vote_timing.tally_end + 10) as u32, leader_1.rest());

    for node in nodes {
        node.logger
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
    }
}

#[allow(dead_code)]
pub fn build_load_config(
    address: &str,
    qr_codes_folder: PathBuf,
    threads_no: usize,
    delay: u64,
    batch_size: usize,
    parameters: Config,
) -> NodeLoadConfig {
    let config = ConfigurationBuilder::duration(parameters.calculate_vote_duration())
        .thread_no(threads_no)
        .step_delay(Duration::from_millis(delay))
        .fetch_limit(250)
        .monitor(Monitor::Progress(100))
        .shutdown_grace_period(Duration::from_secs(60))
        .build();

    NodeLoadConfig {
        batch_size,
        use_v1: false,
        config,
        criterion: Some(100),
        address: address.to_string(),
        qr_codes_folder: Some(qr_codes_folder),
        secrets_folder: None,
        global_pin: "".to_string(),
        reuse_accounts_lazy: false,
        reuse_accounts_early: false,
        read_pin_from_filename: true,
        use_https: false,
        debug: false,
        voting_group: "direct".to_string(),
    }
}

#[allow(dead_code)]
pub fn build_load_config_count(
    address: &str,
    qr_codes_folder: PathBuf,
    threads_no: usize,
    requests_per_thread: u32,
    delay: u64,
    batch_size: usize,
) -> NodeLoadConfig {
    let config = ConfigurationBuilder::requests_per_thread(requests_per_thread)
        .thread_no(threads_no)
        .step_delay(Duration::from_millis(delay))
        .fetch_limit(250)
        .monitor(Monitor::Progress(100))
        .shutdown_grace_period(Duration::from_secs(60))
        .build();

    NodeLoadConfig {
        batch_size,
        use_v1: false,
        config,
        criterion: Some(100),
        address: address.to_string(),
        qr_codes_folder: Some(qr_codes_folder),
        secrets_folder: None,
        global_pin: "".to_string(),
        reuse_accounts_lazy: false,
        reuse_accounts_early: false,
        read_pin_from_filename: true,
        use_https: false,
        debug: false,
        voting_group: "direct".to_string(),
    }
}
