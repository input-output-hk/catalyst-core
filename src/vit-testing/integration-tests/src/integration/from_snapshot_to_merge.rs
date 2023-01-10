use crate::common::iapyx_from_mainnet;
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::CardanoWallet;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::jcli::JCli;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::TallyResult;
use jormungandr_lib::interfaces::{Tally, VotePlanStatus};
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_trigger_service::config::JobParameters;
use std::path::Path;
use std::str::FromStr;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VoteBlockchainTime;
use vitup::config::{ConfigBuilder, DIRECT_VOTING_GROUP, REP_VOTING_GROUP};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;
#[test]
pub fn cip36_and_voting_group_merge() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let discrimination = chain_addr::Discrimination::Production;
    let stake = 10_000;
    let yes_vote = chain_impl_mockchain::vote::Choice::new(0);

    let alice = CardanoWallet::new(stake);
    let bob = CardanoWallet::new(stake);
    let clarice = CardanoWallet::new(stake);

    let david = CardanoWallet::new(500);
    let edgar = CardanoWallet::new(1_000);
    let _fred = CardanoWallet::new(8_000);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_delegator(vec![(&david, 1)]))
        .with(clarice.as_delegator(vec![(&david, 1), (&edgar, 1)]))
        .build();

    let voter_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter_default(&reps)
        .to_voters_hirs();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 10,
    };

    let mut config = ConfigBuilder::default()
        .vote_timing(vote_timing.into())
        .build();
    config
        .initials
        .block0
        .extend_from_external(voter_hir, discrimination);

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

    let mut alice = iapyx_from_mainnet(&alice, &wallet_proxy).unwrap();
    let mut david = iapyx_from_mainnet(&david, &wallet_proxy).unwrap();
    let mut edgar = iapyx_from_mainnet(&edgar, &wallet_proxy).unwrap();

    let voter_proposals = wallet_proxy
        .client()
        .proposals(DIRECT_VOTING_GROUP)
        .unwrap();
    let representative_proposals = wallet_proxy.client().proposals(REP_VOTING_GROUP).unwrap();

    let voter_proposal = &voter_proposals[0];
    let representative_proposal = representative_proposals
        .iter()
        .find(|p| p.proposal.chain_proposal_id == voter_proposal.proposal.chain_proposal_id)
        .expect("cannot find matching proposal between voter and representative");

    alice.vote(voter_proposal, yes_vote).unwrap();

    david.vote(representative_proposal, yes_vote).unwrap();

    edgar.vote(representative_proposal, yes_vote).unwrap();

    let tally_start_date = BlockDate {
        epoch: vote_timing.tally_start,
        slot_id: 0,
    };

    crate::time::wait_for_date(tally_start_date.into(), nodes[0].rest());

    let fragment_sender = FragmentSender::from(&controller.settings().block0);
    let mut committee = controller.wallet("committee_1").unwrap();

    for vote_plan in &controller.defined_vote_plans() {
        fragment_sender
            .send_public_vote_tally(&mut committee, &vote_plan.clone().into(), &nodes[0])
            .unwrap();
    }

    let vote_plans = wallet_proxy
        .client()
        .node_client()
        .vote_plan_statuses()
        .unwrap();
    let vote_plan_statuses_file = testing_directory.child("vote_plan_statuses.yaml");
    write_vote_plans_statuses(vote_plans, vote_plan_statuses_file.path());

    let merged_vote_plans = JCli::new(Path::new("jcli").to_path_buf())
        .votes()
        .tally()
        .merge_results(vote_plan_statuses_file.path())
        .unwrap();

    let merged_vote_plan = merged_vote_plans.get(0).unwrap();

    let voter_proposal_id =
        Hash::from_str(std::str::from_utf8(&voter_proposal.proposal.chain_proposal_id).unwrap())
            .unwrap();

    let merged_proposal = merged_vote_plan
        .proposals
        .iter()
        .find(|p| p.proposal_id == voter_proposal_id)
        .unwrap();
    assert_eq!(merged_proposal.votes_cast, 3);
    assert_eq!(
        merged_proposal.tally,
        Tally::Public {
            result: TallyResult {
                options: 0..2,
                results: vec![15_000, 0]
            }
        }
    );
}

pub fn write_vote_plans_statuses<P: AsRef<Path>>(
    vote_plans_statuses: Vec<VotePlanStatus>,
    path: P,
) {
    use std::io::Write;
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(
        serde_json::to_string(&vote_plans_statuses)
            .unwrap()
            .as_bytes(),
    )
    .unwrap()
}
