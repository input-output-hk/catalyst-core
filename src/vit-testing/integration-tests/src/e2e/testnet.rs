use crate::common::iapyx_from_qr;
use crate::common::registration::do_registration;
use crate::common::snapshot::do_snapshot;
use crate::common::snapshot::wait_for_db_sync;
use crate::Vote;
use assert_fs::TempDir;
use chain_impl_mockchain::header::BlockDate;
use jormungandr_automation::testing::asserts::VotePlanStatusAssert;
use jormungandr_automation::testing::time;
use snapshot_trigger_service::config::JobParameters;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Block0Initials;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

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
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();

    println!("Snapshot: {:?}", snapshot_result);

    let entry = snapshot_result
        .by_address(&result.address().unwrap().into())
        .unwrap()
        .unwrap();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let testing_directory = TempDir::new().unwrap().into_persistent();
    let config = ConfigBuilder::default()
        .slot_duration_in_seconds(2)
        .vote_timing(vote_timing.into())
        .proposals_count(300)
        .voting_power(1)
        .block0_initials(Block0Initials::new_from_external(
            snapshot_result.initials().to_vec(),
        ))
        .private(false)
        .build();

    println!("{:?}", testing_directory.path().to_path_buf());

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[4];
    let mut committee = controller.wallet("committee_1").unwrap();

    // start wallets
    let mut alice = iapyx_from_qr(&result.qr_code(), &result.pin(), &wallet_proxy).unwrap();

    let fund1_vote_plan = &controller.defined_vote_plans()[0];
    let fund2_vote_plan = &controller.defined_vote_plans()[1];

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

    let fragment_sender = FragmentSender::from(&controller.settings().block0);

    fragment_sender
        .send_public_vote_tally(&mut committee, &fund1_vote_plan.clone().into(), wallet_node)
        .unwrap();

    fragment_sender
        .send_public_vote_tally(&mut committee, &fund2_vote_plan.clone().into(), wallet_node)
        .unwrap();

    vote_timing.wait_for_tally_end(leader_1.rest());

    let vote_plans = leader_1.rest().vote_plan_statuses().unwrap();
    vote_plans.assert_proposal_tally(fund1_vote_plan.id(), 0, vec![u64::from(entry.value), 0]);
    vote_plans.assert_proposal_tally(fund2_vote_plan.id(), 0, vec![u64::from(entry.value), 0]);
}
