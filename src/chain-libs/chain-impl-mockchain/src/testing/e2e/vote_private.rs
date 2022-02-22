use crate::testing::decrypt_tally;
use crate::testing::TestGen;
use crate::testing::VoteTestGen;
use crate::tokens::name::TokenName;
use crate::tokens::name::TOKEN_NAME_MAX_SIZE;
use crate::vote::VoteError::AlreadyVoted;
use crate::vote::VotePlanLedgerError::VoteError;
use crate::{
    certificate::VotePlan,
    fee::LinearFee,
    header::BlockDate,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, proposal, vote_plan, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
    vote::{Choice, PayloadType},
};
use imhamt::UpdateError::ValueCallbackError;

const ALICE: &str = "Alice";
const STAKE_POOL: &str = "stake_pool";
const VOTE_PLAN: &str = "fund1";

const MEMBERS_NO: usize = 3;
const THRESHOLD: usize = 2;

#[test]
pub fn private_vote_cast_action_transfer_to_rewards_all_shares() {
    let mut rng = TestGen::rand();
    let favorable = Choice::new(1);
    let members = VoteTestGen::committee_members_manager(MEMBERS_NO, THRESHOLD);

    let voting_token = TokenName::try_from(vec![0u8; TOKEN_NAME_MAX_SIZE]).unwrap();

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new()
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000)),
        )
        .with_initials(vec![wallet(ALICE)
            .with(1_000)
            .with_token(voting_token, 1_000)
            .owns(STAKE_POOL)
            .committee_member()])
        .with_vote_plans(vec![vote_plan(VOTE_PLAN)
            .owner(ALICE)
            .consecutive_epoch_dates()
            .payload_type(PayloadType::Private)
            .committee_keys(members.members_keys())
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
        .cast_vote_private(
            &alice,
            &vote_plan,
            &proposal.id(),
            favorable,
            &mut ledger,
            &mut rng,
        )
        .unwrap();
    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .encrypted_tally(&alice, &vote_plan, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    let vote_plans = ledger.ledger.active_vote_plans();
    let vote_plan_status = vote_plans
        .iter()
        .find(|c_vote_plan| {
            let vote_plan: VotePlan = vote_plan.clone().into();
            c_vote_plan.id == vote_plan.to_id()
        })
        .unwrap();

    let shares = decrypt_tally(vote_plan_status, &members).unwrap();

    controller
        .tally_vote_private(&alice, &vote_plan, shares, &mut ledger)
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
pub fn shouldnt_panic_when_no_initial_tokens_and_no_votes() {
    let mut rng = TestGen::rand();
    let favorable = Choice::new(1);
    let members = VoteTestGen::committee_members_manager(MEMBERS_NO, THRESHOLD);

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
            .payload_type(PayloadType::Private)
            .committee_keys(members.members_keys())
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
        .cast_vote_private(
            &alice,
            &vote_plan,
            &proposal.id(),
            favorable,
            &mut ledger,
            &mut rng,
        )
        .unwrap();
    alice.confirm_transaction();

    ledger.fast_forward_to(BlockDate {
        epoch: 1,
        slot_id: 1,
    });

    controller
        .encrypted_tally(&alice, &vote_plan, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    let vote_plans = ledger.ledger.active_vote_plans();
    let vote_plan_status = vote_plans
        .iter()
        .find(|c_vote_plan| {
            let vote_plan: VotePlan = vote_plan.clone().into();
            c_vote_plan.id == vote_plan.to_id()
        })
        .unwrap();

    decrypt_tally(vote_plan_status, &members).unwrap();
}

#[test]
#[should_panic]
pub fn private_vote_plan_without_keys() {
    let committee_keys = vec![];

    let (_ledger, _controller) = prepare_scenario()
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
            .payload_type(PayloadType::Private)
            .committee_keys(committee_keys)
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(100),
            )])
        .build()
        .unwrap();
}

#[test]
pub fn vote_on_same_proposal() {
    let mut rng = TestGen::rand();
    let favorable = Choice::new(1);

    let members = VoteTestGen::committee_members_manager(MEMBERS_NO, THRESHOLD);

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
            .payload_type(PayloadType::Private)
            .committee_keys(members.members_keys())
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
        .cast_vote_private(
            &alice,
            &vote_plan,
            &proposal.id(),
            favorable,
            &mut ledger,
            &mut rng,
        )
        .unwrap();

    alice.confirm_transaction();

    let inner_vote_plan: VotePlan = vote_plan.clone().into();

    assert_eq!(
        controller
            .cast_vote_private(
                &alice,
                &vote_plan,
                &proposal.id(),
                favorable,
                &mut ledger,
                &mut rng
            )
            .err()
            .unwrap(),
        crate::ledger::ledger::Error::VotePlan(VoteError {
            id: inner_vote_plan.to_id(),
            reason: ValueCallbackError(AlreadyVoted)
        })
    );
}

#[test]
pub fn vote_on_different_proposal() {
    let mut rng = TestGen::rand();
    let favorable = Choice::new(1);

    let members = VoteTestGen::committee_members_manager(MEMBERS_NO, THRESHOLD);

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
            .payload_type(PayloadType::Private)
            .committee_keys(members.members_keys())
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(100),
            )
            .with_proposal(
                proposal(VoteTestGen::external_proposal_id())
                    .options(3)
                    .action_transfer_to_rewards(100),
            )])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();

    let vote_plan = controller.vote_plan(VOTE_PLAN).unwrap();
    let first_proposal = vote_plan.proposal(0);
    let second_proposal = vote_plan.proposal(1);

    controller
        .cast_vote_private(
            &alice,
            &vote_plan,
            &first_proposal.id(),
            favorable,
            &mut ledger,
            &mut rng,
        )
        .unwrap();

    alice.confirm_transaction();

    assert!(controller
        .cast_vote_private(
            &alice,
            &vote_plan,
            &second_proposal.id(),
            favorable,
            &mut ledger,
            &mut rng
        )
        .is_ok());
}
