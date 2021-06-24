use crate::common::registration::do_registration;
use crate::common::snapshot::do_snapshot;
use crate::common::snapshot::wait_for_db_sync;
use crate::common::{
    asserts::VotePlanStatusAssert, vitup_setup, wait_until_folder_contains_all_qrs, Error, Vote,
    VoteTiming,
};
use assert_fs::TempDir;
use chain_impl_mockchain::header::BlockDate;
use iapyx::Protocol;
use jormungandr_testing_utils::testing::node::time;
use registration_service::context::State;
use snapshot_trigger_service::{client::rest::SnapshotRestClient, config::JobParameters};
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Initials;
use vitup::scenario::network::setup_network;
use vitup::setup::start::QuickVitBackendSettingsBuilder;
const GRACE_PERIOD_FOR_SNAPSHOT: u64 = 300;

#[test]
pub fn e2e_flow_using_voter_registration_local_vitup_and_iapyx() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let result = do_registration(&temp_dir);

    result.assert_status_is_finished();
    result.assert_qr_equals_to_sk();

    println!("Registraton Result: {:?}", result);

    let job_param = JobParameters {
        slot_no: Some(result.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
        threshold: 1_000_000,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(&temp_dir, job_param).unwrap();

    println!("Snapshot: {:?}", snapshot_result);

    let entry = snapshot_result
        .by_address(&result.address())
        .unwrap()
        .unwrap();

    let endpoint = "127.0.0.1:8080";
    let vote_timing = VoteTiming::new(0, 1, 2);
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .vote_start_epoch(vote_timing.vote_start)
        .tally_start_epoch(vote_timing.tally_start)
        .tally_end_epoch(vote_timing.tally_end)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(30)
        .proposals_count(300)
        .voting_power(1)
        .initials(Initials::new_from_external(
            snapshot_result.initials().unwrap(),
        ))
        .private(false);

    println!("{:?}", testing_directory.path().to_path_buf());

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
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

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[4];
    let mut committee = controller.wallet("committee_1").unwrap();

    // start wallets
    let mut alice = vit_controller
        .iapyx_wallet_from_qr(&result.qr_code(), &result.pin(), &wallet_proxy)
        .unwrap();

    let fund1_vote_plan = &controller.vote_plans()[0];
    let fund2_vote_plan = &controller.vote_plans()[1];

    alice
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    alice
        .vote_for(fund2_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.rest());

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &fund1_vote_plan.clone().into(), wallet_node)
        .unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &fund2_vote_plan.clone().into(), wallet_node)
        .unwrap();

    vote_timing.wait_for_tally_end(leader_1.rest());

    let vote_plans = leader_1.vote_plans().unwrap();
    vote_plans.assert_all_proposals_are_tallied();
    vote_plans.assert_proposal_tally(fund1_vote_plan.id(), 0, vec![0, u64::from(entry.value), 0]);
    vote_plans.assert_proposal_tally(fund2_vote_plan.id(), 0, vec![0, u64::from(entry.value), 0]);

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.shutdown().unwrap();
    }
    controller.finalize();
}
