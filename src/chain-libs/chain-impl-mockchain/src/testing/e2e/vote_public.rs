use crate::testing::VoteTestGen;
use crate::{
    fee::{LinearFee, PerCertificateFee, PerVoteCertificateFee},
    header::BlockDate,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, proposal, vote_plan, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
    vote::Choice,
};
use core::num::NonZeroU64;

const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const STAKE_POOL: &str = "stake_pool";
const VOTE_PLAN: &str = "fund1";

#[test]
pub fn vote_cast_action_transfer_to_rewards() {
    let favorable = Choice::new(1);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote_public(&alice, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(1100));
}

#[test]
pub fn vote_cast_action_action_parameters_no_op() {
    let favorable = Choice::new(1);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_parameters_no_op(),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote_public(&alice, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(1000));
}

#[test]
pub fn vote_cast_tally_50_percent() {
    let _blank = Choice::new(0);
    let favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![
            wallet(ALICE)
                .with(1_000)
                .owns(STAKE_POOL)
                .committee_member(),
            wallet(BOB)
                .with(1_000)
                .delegates_to(STAKE_POOL)
                .committee_member(),
        ])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let mut bob = controller.wallet(BOB).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote_public(&bob, &vote_plan, &proposal.id(), rejection, &mut ledger)
        .unwrap();
    bob.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote_public(&bob, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(1100));
}

#[test]
pub fn vote_cast_tally_50_percent_unsuccesful() {
    let _blank = Choice::new(0);
    let _favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![
            wallet(ALICE)
                .with(1_000)
                .owns(STAKE_POOL)
                .committee_member(),
            wallet(BOB)
                .with(1_000)
                .delegates_to(STAKE_POOL)
                .committee_member(),
        ])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let mut bob = controller.wallet(BOB).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), rejection, &mut ledger)
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote_public(&bob, &vote_plan, &proposal.id(), rejection, &mut ledger)
        .unwrap();
    bob.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote_public(&bob, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(1000));
}

#[test]
#[should_panic]
pub fn vote_plan_creates_by_non_committee_member() {
    let _ = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE).with(1_000).owns(STAKE_POOL)])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build();
}

#[test]
pub fn vote_cast_by_non_committe_member() {
    let _blank = Choice::new(0);
    let favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![
            wallet(ALICE)
                .with(1_000)
                .owns(STAKE_POOL)
                .committee_member(),
            wallet(BOB).with(1_000).delegates_to(STAKE_POOL),
        ])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build()
        .unwrap();

    let bob = controller.wallet(BOB).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&bob, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();
    assert!(controller
        .cast_vote_public(&bob, &vote_plan, &proposal.id(), rejection, &mut ledger,)
        .is_err());
}

#[test]
pub fn vote_on_same_proposal() {
    let _blank = Choice::new(0);
    let favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();

    alice.confirm_transaction();

    assert!(controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), rejection, &mut ledger)
        .is_err());
}

#[test]
pub fn vote_on_different_proposal() {
    let _blank = Choice::new(0);
    let favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_rewards_add(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let first_proposal = vote_plan.proposal(0);
    let second_proposal = vote_plan.proposal(1);

    controller
        .cast_vote_public(
            &alice,
            &vote_plan,
            &first_proposal.id(),
            favorable,
            &mut ledger,
        )
        .unwrap();

    alice.confirm_transaction();

    assert!(controller
        .cast_vote_public(
            &alice,
            &vote_plan,
            &second_proposal.id(),
            rejection,
            &mut ledger
        )
        .is_ok());
}

#[test]
pub fn votes_with_fees() {
    let favorable = Choice::new(1);
    let rewards_add = 100;
    let initial_rewards = 1000;
    let mut fees = LinearFee::new(1, 1, 1);

    let cert_fees = PerCertificateFee::new(
        Some(NonZeroU64::new(1).unwrap()),
        Some(NonZeroU64::new(1).unwrap()),
        Some(NonZeroU64::new(1).unwrap()),
    );
    fees.per_certificate_fees(cert_fees);

    let vote_fees = PerVoteCertificateFee::new(
        Some(NonZeroU64::new(1).unwrap()),
        Some(NonZeroU64::new(1).unwrap()),
    );
    fees.per_vote_certificate_fees(vote_fees);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(fees)
                .with_rewards(Value(initial_rewards)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(rewards_add),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let proposal = vote_plan.proposal(0);
    let total_ada_before = ledger.total_funds();

    controller
        .cast_vote_public(&alice, &vote_plan, &proposal.id(), favorable, &mut ledger)
        .unwrap();

    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote_public(&alice, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(initial_rewards + rewards_add));

    let expected_ada_after = total_ada_before.saturating_add(Value(rewards_add));

    LedgerStateVerifier::new(ledger.into())
        .info("total value is the same")
        .total_value_is(&expected_ada_after);
}

#[test]
pub fn voting_consistency() {
    let favorable = Choice::new(1);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new())
        .with_initials(vec![wallet(ALICE).with(1_000).committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_off_chain(),
            )
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_off_chain(),
            )
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_off_chain(),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();

    controller
        .cast_vote_public(
            &alice,
            &vote_plan,
            &vote_plan.proposal(0).id(),
            favorable,
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote_public(
            &alice,
            &vote_plan,
            &vote_plan.proposal(1).id(),
            favorable,
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote_public(
            &alice,
            &vote_plan,
            &vote_plan.proposal(2).id(),
            favorable,
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();
    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    LedgerStateVerifier::new(ledger.into())
        .info("votes history")
        .votes()
        .gvien_wallet(&alice)
        .for_vote_plan(&vote_plan)
        .votes_were_casted_on_proposals(vec![0u8, 1u8, 2u8]);
}
