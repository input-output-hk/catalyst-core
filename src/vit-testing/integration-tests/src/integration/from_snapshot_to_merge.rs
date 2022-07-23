use crate::common::iapyx_from_mainnet;
use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use catalyst_toolbox::snapshot::voting_group::RepsVotersAssigner;
use catalyst_toolbox::snapshot::Delegations;
use chain_impl_mockchain::block::BlockDate;
use fraction::Fraction;
use jormungandr_automation::jcli::JCli;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::TallyResult;
use jormungandr_lib::interfaces::{Tally, VotePlanStatus};
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_trigger_service::config::ConfigurationBuilder;
use snapshot_trigger_service::config::JobParameters;
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

const DIRECT_VOTING_GROUP: &str = "direct";
const REP_VOTING_GROUP: &str = "dreps";

#[test]
pub fn cip36_and_voting_group_merge() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let discrimination = chain_addr::Discrimination::Production;
    let stake = 10_000;
    let yes_vote = chain_impl_mockchain::vote::Choice::new(0);

    let alice_voter = MainnetWallet::new(stake);
    let bob_voter = MainnetWallet::new(stake);
    let clarice_voter = MainnetWallet::new(stake);

    let david_representative = MainnetWallet::new(500);
    let edgar_representative = MainnetWallet::new(1_000);
    let fred_representative = MainnetWallet::new(8_000);

    let mut reps = HashSet::new();
    reps.insert(edgar_representative.catalyst_public_key());
    reps.insert(david_representative.catalyst_public_key());
    reps.insert(fred_representative.catalyst_public_key());

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network);
    bob_voter
        .send_voting_registration(Delegations::New(vec![(
            david_representative.catalyst_public_key(),
            1,
        )]))
        .to(&mut mainnet_network);
    clarice_voter
        .send_voting_registration(Delegations::New(vec![
            (david_representative.catalyst_public_key(), 1),
            (edgar_representative.catalyst_public_key(), 1),
        ]))
        .to(&mut mainnet_network);

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let assigner = RepsVotersAssigner::new_from_repsdb(
        DIRECT_VOTING_GROUP.to_string(),
        REP_VOTING_GROUP.to_string(),
        reps,
    )
    .unwrap();

    let snapshot_service = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap();

    let voter_hir = snapshot_service
        .snapshot(
            JobParameters::fund("fund9"),
            450u64,
            Fraction::from(1u64),
            &assigner,
        )
        .to_voter_hir();

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

    let mut alice = iapyx_from_mainnet(&alice_voter, &wallet_proxy).unwrap();
    let mut david = iapyx_from_mainnet(&david_representative, &wallet_proxy).unwrap();
    let mut edgar = iapyx_from_mainnet(&edgar_representative, &wallet_proxy).unwrap();

    let voter_proposals = wallet_proxy
        .client()
        .proposals(DIRECT_VOTING_GROUP)
        .unwrap();
    let representative_proposals = wallet_proxy.client().proposals(REP_VOTING_GROUP).unwrap();

    let voter_proposal = &voter_proposals[0];
    let representative_proposal = representative_proposals
        .iter()
        .find(|p| p.proposal.chain_proposal_id == voter_proposal.proposal.chain_proposal_id)
        .unwrap();

    assert_eq!(
        voter_proposal.proposal.chain_proposal_id,
        representative_proposal.proposal.chain_proposal_id
    );

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
                results: vec![30_000, 0]
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
