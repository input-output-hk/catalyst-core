use crate::common::{load::build_load_config, vitup_setup};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use catalyst_toolbox::recovery::Replay;
use iapyx::NodeLoad;
use jcli_lib::utils::output_file::OutputFile;
use jcli_lib::utils::output_format::{FormatVariant, OutputFormat};
use jormungandr_lib::interfaces::VotePlanStatus;
use jormungandr_testing_utils::testing::block0::get_block;
use jortestkit::measurement::Status;
use serde_json;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::VitBackendSettingsBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::scenario::network::setup_network;

#[test]
pub fn persistent_log_contains_all_sent_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";

    let version = "2.0";
    let no_of_threads = 2;
    let batch_size = 10;
    let no_of_wallets = 1;

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 60,
    };

    let mut quick_setup = VitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .slot_duration_in_seconds(2)
        .vote_timing(vote_timing.into())
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

    let config = build_load_config(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        batch_size,
        setup_parameters,
    );
    let iapyx_load = NodeLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    vote_timing.wait_for_tally_start(nodes.get(0).unwrap().rest());

    let mut committee = controller.wallet("committee_1").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(0).unwrap())
        .unwrap();

    let offline_tally = testing_directory.child("offline_tally.json");

    let replay = Replay::new(
        get_block(
            controller
                .block0_file()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .unwrap()
        .to_block(),
        controller.working_directory().path().join("persistent_log"),
        OutputFile::from(offline_tally.path().to_path_buf()),
        OutputFormat::from(FormatVariant::Json),
    );
    replay.exec().unwrap();
    let offline_voteplan_status: Vec<VotePlanStatus> =
        serde_json::from_str(&jortestkit::file::read_file(offline_tally.path())).unwrap();

    let live_voteplan_status = wallet_proxy.client().vote_plan_statuses().unwrap();

    vit_station.shutdown();
    wallet_proxy.shutdown();

    for node in nodes {
        node.logger()
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
        node.shutdown().unwrap();
    }

    controller.finalize();

    assert_eq!(live_voteplan_status, offline_voteplan_status);
}
