use crate::config::ConfigParam;
use crate::fragment::ConfigParams;
use crate::key::EitherEd25519SecretKey;
use crate::{
    certificate::{UpdateProposal, UpdateVote},
    header::BlockDate,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, wallet},
        TestGen,
    },
};
use chain_addr::Discrimination;

const ALICE: &str = "ALICE";

#[test]
pub fn update_slot_duration() {
    let initial_slot_duration = 10;
    let final_slot_duration = 100;
    let leader_pair = TestGen::leader_pair();
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_slot_duration(initial_slot_duration)
                .with_discrimination(Discrimination::Test)
                .with_leaders(&[leader_pair.id()]),
        )
        .with_initials(vec![wallet(ALICE)
            .key(EitherEd25519SecretKey::Normal(leader_pair.key()))
            .with(1_000)])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let mut config_params = ConfigParams::new();
    config_params.push(ConfigParam::SlotDuration(final_slot_duration));

    let update_proposal = UpdateProposal::new(config_params, leader_pair.id());

    let proposal_id = controller
        .update_proposal(&alice, update_proposal, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    controller
        .update_vote(
            &alice,
            UpdateVote::new(proposal_id, leader_pair.id()),
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 0,
        slot_id: 1,
    });

    assert_eq!(initial_slot_duration, ledger.settings().slot_duration);

    assert!(ledger
        .apply_empty_bft_block_with_date(
            &leader_pair,
            BlockDate {
                epoch: 1,
                slot_id: 0,
            }
        )
        .is_ok());

    assert_eq!(final_slot_duration, ledger.settings().slot_duration);
}
