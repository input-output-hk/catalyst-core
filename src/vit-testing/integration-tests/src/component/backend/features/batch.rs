use crate::common::iapyx_from_secret_key;
use assert_fs::TempDir;
use chain_impl_mockchain::vote::Choice;
use jormungandr_automation::testing::time;
use jormungandr_lib::interfaces::FragmentStatus;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initial, Block0Initials, ConfigBuilder};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

const PIN: &str = "1234";
const ALICE: &str = "alice";

#[test]
pub fn transactions_are_send_between_nodes_with_correct_order() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let batch_size = 10;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 60,
    };

    let role = Default::default();
    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![Block0Initial::Wallet {
            name: ALICE.to_string(),
            funds: 10_000,
            pin: PIN.to_string(),
            role,
        }]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(300)
        .voting_power(31_000)
        .private(true)
        .build();

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

    let secret = testing_directory.path().join("alice");
    let mut alice = iapyx_from_secret_key(secret, &wallet_proxy).unwrap();

    let proposals = alice.proposals(&role.to_string()).unwrap();
    let votes_data = proposals
        .iter()
        .take(batch_size)
        .map(|proposal| (proposal, Choice::new(0)))
        .collect();

    let fragment_ids = alice
        .votes_batch(votes_data)
        .unwrap()
        .iter()
        .map(|item| item.to_string())
        .collect();

    time::wait_for_epoch(1, nodes[0].rest());

    let statuses = nodes[0].rest().fragments_statuses(fragment_ids).unwrap();
    assert!(statuses
        .iter()
        .all(|(_, status)| matches!(status, FragmentStatus::InABlock { .. })));
}
