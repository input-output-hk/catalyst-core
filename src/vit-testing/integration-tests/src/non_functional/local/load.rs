use crate::common::{load::build_load_config, load::private_vote_test_scenario};
use assert_fs::TempDir;
use iapyx::NodeLoad;
use jortestkit::measurement::Status;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn load_test_public_100_000_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";
    let no_of_threads = 10;
    let no_of_wallets = 40_000;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 100,
        tally_end: 102,
        slots_per_epoch: 60,
    };

    let config = ConfigBuilder::default()
        .block0_initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(300)
        .voting_power(31_000)
        .private(false)
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    let (nodes, _vit_station, _wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("qr-codes");

    let load_config = build_load_config(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        100,
        1,
        config.clone(),
    );
    let iapyx_load = NodeLoad::new(load_config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    vote_timing.wait_for_tally_start(nodes.get(0).unwrap().rest());

    let mut committee = controller.wallet("committee").unwrap();
    let vote_plan = controller
        .defined_vote_plan(&config.data.current_fund.fund_info.fund_name)
        .unwrap();

    let fragment_sender = FragmentSender::from(&controller.settings().block0);

    fragment_sender
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(0).unwrap())
        .unwrap();

    for node in nodes {
        node.logger
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
    }
}

#[test]
pub fn load_test_private_pesimistic() {
    let no_of_threads = 10;
    let endpoint = "127.0.0.1:8080";
    let no_of_wallets = 8_000;

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 11,
        tally_end: 12,
        slots_per_epoch: 3,
    };

    let config = ConfigBuilder::default()
        .block0_initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(20)
        .proposals_count(250)
        .voting_power(31_000)
        .private(true)
        .build();

    private_vote_test_scenario(config, endpoint, no_of_threads, 1);
}

#[test]
pub fn load_test_private_optimistic() {
    let no_of_threads = 10;
    let no_of_wallets = 20_000;
    let endpoint = "127.0.0.1:8080";
    let vote_timing = VoteBlockchainTime {
        vote_start: 6,
        tally_start: 11,
        tally_end: 12,
        slots_per_epoch: 180,
    };

    let config = ConfigBuilder::default()
        .block0_initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(20)
        .proposals_count(500)
        .voting_power(31_000)
        .private(true)
        .build();

    private_vote_test_scenario(config, endpoint, no_of_threads, 1);
}
