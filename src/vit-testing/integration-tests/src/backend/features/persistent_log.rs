use crate::common::load::build_load_config_count;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use catalyst_toolbox::recovery::Replay;
use iapyx::NodeLoad;
use jcli_lib::utils::output_file::OutputFile;
use jcli_lib::utils::output_format::{FormatVariant, OutputFormat};
use jormungandr_automation::testing::block0::get_block;
use jormungandr_lib::interfaces::VotePlanStatus;
use jortestkit::measurement::Status;
use serde_json;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::VitBackendSettingsBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn persistent_log_contains_all_sent_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "http://127.0.0.1:8080";
    let no_of_threads = 1;
    let number_requests_to_per_threads = 50;
    let batch_size = 20;
    let no_of_wallets = 100;

    let vote_timing = VoteBlockchainTime {
        vote_start: 1,
        tally_start: 2,
        tally_end: 3,
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

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut controller, vit_parameters, network_params, fund_name) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());

    let (nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("qr-codes");

    let config = build_load_config_count(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        number_requests_to_per_threads,
        1000,
        batch_size,
    );
    let iapyx_load = NodeLoad::new(config);

    vote_timing.wait_for_vote_start(nodes.get(0).unwrap().rest());

    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    vote_timing.wait_for_tally_start(nodes.get(0).unwrap().rest());

    let mut committee = controller.wallet("committee_1").unwrap();
    let vote_plan = controller.defined_vote_plan(&fund_name).unwrap();

    let fragment_sender = FragmentSender::from(&controller.settings().block0);

    fragment_sender
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

    for node in nodes {
        node.logger
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
    }
    assert_eq!(live_voteplan_status, offline_voteplan_status);
}
