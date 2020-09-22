use crate::testing::VoteTestGen;
use crate::{
    fee::LinearFee,
    header::BlockDate,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, proposal, vote_plan, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
    vote::Choice,
};

const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const STAKE_POOL: &str = "stake_pool";
const VOTE_PLAN: &str = "fund1";

#[test]
pub fn vote_cast_tally_50_percent() {
    let _blank = Choice::new(0);
    let favorable = Choice::new(1);
    let rejection = Choice::new(2);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
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
        .cast_vote(
            &alice,
            &vote_plan,
            &proposal.id(),
            favorable.clone(),
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote(
            &bob,
            &vote_plan,
            &proposal.id(),
            rejection.clone(),
            &mut ledger,
        )
        .unwrap();
    bob.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote(&bob, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
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
            ConfigBuilder::new(0)
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
        .cast_vote(
            &alice,
            &vote_plan,
            &proposal.id(),
            rejection.clone(),
            &mut ledger,
        )
        .unwrap();
    alice.confirm_transaction();
    controller
        .cast_vote(
            &bob,
            &vote_plan,
            &proposal.id(),
            rejection.clone(),
            &mut ledger,
        )
        .unwrap();
    bob.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .tally_vote(&bob, &vote_plan, &mut ledger)
        .unwrap();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    ledger.apply_protocol_changes().unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
        .info("rewards pot is increased")
        .pots()
        .has_remaining_rewards_equals_to(&Value(1000));
}

#[test]
#[should_panic]
pub fn vote_plan_creates_by_non_committee_member() {
    let _ = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
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
            ConfigBuilder::new(0)
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
        .cast_vote(
            &bob,
            &vote_plan,
            &proposal.id(),
            favorable.clone(),
            &mut ledger,
        )
        .unwrap();
    assert!(controller
        .cast_vote(
            &bob,
            &vote_plan,
            &proposal.id(),
            rejection.clone(),
            &mut ledger
        )
        .is_err());
}
