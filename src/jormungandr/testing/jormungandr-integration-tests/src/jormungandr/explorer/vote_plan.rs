use crate::startup::SingleNodeTestBootstrapper;
use assert_fs::TempDir;
use chain_addr::Discrimination;
use chain_core::property::BlockDate as propertyBlockDate;
use chain_impl_mockchain::{
    block::BlockDate,
    certificate::{VoteAction, VotePlan},
    chaintypes::ConsensusType,
    ledger::governance::TreasuryGovernanceAction,
    testing::data::Wallet as chainWallet,
    tokens::minting_policy::MintingPolicy,
    value::Value,
    vote::Choice,
};
use jormungandr_automation::{
    jormungandr::{
        explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier},
        Block0ConfigurationBuilder,
    },
    testing::{
        time::{get_current_date, wait_for_date},
        VotePlanBuilder,
    },
};
use jormungandr_lib::interfaces::{Initial, InitialToken, KesUpdateSpeed};
use rand_core::OsRng;
use std::{collections::HashMap, iter, time::Duration};
use thor::{
    vote_plan_cert, CommitteeDataManager, FragmentBuilder, FragmentSender, FragmentSenderSetup,
    FragmentVerifier, Wallet,
};

const INITIAL_FUND_PER_WALLET_1: u64 = 1_000_000;
const INITIAL_FUND_PER_WALLET_2: u64 = 2_000_000;
const INITIAL_TOKEN_PER_WALLET_1: u64 = 1_000_000;
const INITIAL_TOKEN_PER_WALLET_2: u64 = 2_000_000;
const INITIAL_TREASURY: u64 = 1000;
const REWARD_INCREASE: u64 = 10;
const SLOTS_PER_EPOCH: u32 = 20;
const SLOT_DURATION: u8 = 2;

const VOTE_PLAN_QUERY_COMPLEXITY_LIMIT: u64 = 100;
const VOTE_PLAN_QUERY_DEPTH_LIMIT: u64 = 30;
const VOTE_FOR_MARIO: u8 = 0;
const VOTE_FOR_LUIGI: u8 = 1;
const VOTE_FOR_ANTONIO: u8 = 2;

#[test]
pub fn explorer_vote_plan_not_existing() {
    let temp_dir = TempDir::new().unwrap();
    let alice = Wallet::default();
    let proposals = vec![VOTE_FOR_ANTONIO];
    let proposal_count = proposals.len();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(proposal_count)
        .vote_start(BlockDate::from_epoch_slot_id(0, 0))
        .tally_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_end(BlockDate::from_epoch_slot_id(1, 10))
        .public()
        .build();

    let config = Block0ConfigurationBuilder::default()
        .with_utxos(vec![alice.to_initial_fund(INITIAL_FUND_PER_WALLET_1)])
        .with_token(InitialToken {
            token_id: vote_plan.voting_token().clone().into(),
            policy: MintingPolicy::new().into(),
            to: vec![alice.to_initial_token(INITIAL_FUND_PER_WALLET_1)],
        })
        .with_committees(&[alice.to_committee_id()])
        .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
        .with_treasury(INITIAL_TREASURY.into());

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .with_block0_config(config)
        .build()
        .start_node(temp_dir)
        .unwrap();

    let params = ExplorerParams::new(
        VOTE_PLAN_QUERY_COMPLEXITY_LIMIT,
        VOTE_PLAN_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.data.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    assert!(
        query_response.errors.is_some(),
        "{:?}",
        query_response.errors.unwrap()
    );

    assert!(
        &query_response
            .errors
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .message
            .contains("not found"),
        "{:?}",
        query_response.errors.unwrap()
    );
}

#[test]
pub fn explorer_vote_plan_public_flow_test() {
    let temp_dir = TempDir::new().unwrap();
    let alice = Wallet::default();
    let bob = Wallet::default();
    let mut voters = vec![alice, bob];
    let proposals = vec![VOTE_FOR_MARIO, VOTE_FOR_LUIGI, VOTE_FOR_ANTONIO];
    let proposal_count = proposals.len();
    let yes_choice = Choice::new(1);
    let no_choice = Choice::new(0);
    let mut vote_for_mario: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_luigi: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_antonio: Vec<(chainWallet, Choice)> = Vec::new();
    let mut proposal_votes: HashMap<String, Vec<(chainWallet, Choice)>> = HashMap::new();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(proposal_count)
        .vote_start(BlockDate::from_epoch_slot_id(0, 0))
        .tally_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_end(BlockDate::from_epoch_slot_id(1, 10))
        .public()
        .build();

    let vote_plan_cert = Initial::Cert(
        vote_plan_cert(
            &voters[0],
            BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            &vote_plan,
        )
        .into(),
    );

    let config = Block0ConfigurationBuilder::default()
        .with_utxos(vec![
            voters[0].to_initial_fund(INITIAL_FUND_PER_WALLET_1),
            voters[1].to_initial_fund(INITIAL_FUND_PER_WALLET_2),
        ])
        .with_token(InitialToken {
            token_id: vote_plan.voting_token().clone().into(),
            policy: MintingPolicy::new().into(),
            to: vec![
                voters[0].to_initial_token(INITIAL_TOKEN_PER_WALLET_1),
                voters[1].to_initial_token(INITIAL_TOKEN_PER_WALLET_2),
            ],
        })
        .with_committees(&[voters[0].to_committee_id()])
        .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
        .with_certs(vec![vote_plan_cert])
        .with_treasury(INITIAL_TREASURY.into());

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .with_block0_config(config)
        .build()
        .start_node(temp_dir)
        .unwrap();

    let transaction_sender = FragmentSender::from_settings(
        &jormungandr.rest().settings().unwrap(),
        BlockDate {
            epoch: 3,
            slot_id: 0,
        }
        .into(),
        FragmentSenderSetup::resend_3_times(),
    );

    let params = ExplorerParams::new(
        VOTE_PLAN_QUERY_COMPLEXITY_LIMIT,
        VOTE_PLAN_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    //1.Vote plan started
    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    assert!(vote_plan.can_vote(get_current_date(&mut jormungandr.rest()).into()));

    //2. Voting
    transaction_sender
        .send_vote_cast(
            &mut voters[0],
            &vote_plan,
            VOTE_FOR_ANTONIO,
            &yes_choice,
            &jormungandr,
        )
        .unwrap();

    vote_for_antonio.push((chainWallet::from(voters[0].clone()), yes_choice));

    transaction_sender
        .send_vote_cast(
            &mut voters[1],
            &vote_plan,
            VOTE_FOR_ANTONIO,
            &yes_choice,
            &jormungandr,
        )
        .unwrap();

    vote_for_antonio.push((chainWallet::from(voters[1].clone()), yes_choice));

    transaction_sender
        .send_vote_cast(
            &mut voters[0],
            &vote_plan,
            VOTE_FOR_MARIO,
            &no_choice,
            &jormungandr,
        )
        .unwrap();

    vote_for_mario.push((chainWallet::from(voters[0].clone()), no_choice));

    transaction_sender
        .send_vote_cast(
            &mut voters[1],
            &vote_plan,
            VOTE_FOR_LUIGI,
            &no_choice,
            &jormungandr,
        )
        .unwrap();

    vote_for_luigi.push((chainWallet::from(voters[1].clone()), no_choice));

    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_MARIO as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_mario,
    );
    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_ANTONIO as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_antonio,
    );
    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_LUIGI as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_luigi,
    );

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    wait_for_date(vote_plan.vote_end().into(), jormungandr.rest());

    //3.Start talling
    let mempool_check = transaction_sender
        .send_public_vote_tally(&mut voters[0], &vote_plan, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mempool_check,
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    wait_for_date(vote_plan.committee_end().into(), jormungandr.rest());

    //4. End talling
    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );
}

#[test]
pub fn explorer_vote_plan_private_flow_test() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let yes_choice = Choice::new(1);
    let no_choice = Choice::new(2);
    let threshold = 1;
    let mut rng = OsRng;
    let alice = Wallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
    let bob = Wallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
    let mut voters = vec![alice, bob];
    let proposals = vec![VOTE_FOR_MARIO, VOTE_FOR_LUIGI, VOTE_FOR_ANTONIO];
    let proposal_count = proposals.len();
    let private_vote_committee_data_manager =
        CommitteeDataManager::private(&mut OsRng, vec![(voters[0].account_id())], threshold);
    let mut vote_for_mario: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_luigi: Vec<(chainWallet, Choice)> = Vec::new();
    let vote_for_antonio: Vec<(chainWallet, Choice)> = Vec::new();
    let mut proposal_votes: HashMap<String, Vec<(chainWallet, Choice)>> = HashMap::new();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(proposal_count)
        .action_type(VoteAction::Treasury {
            action: TreasuryGovernanceAction::TransferToRewards {
                value: Value(REWARD_INCREASE),
            },
        })
        .private()
        .vote_start(BlockDate::from_epoch_slot_id(0, 0))
        .tally_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_end(BlockDate::from_epoch_slot_id(1, 10))
        .member_public_keys(private_vote_committee_data_manager.member_public_keys())
        .options_size(3)
        .build();

    let vote_plan_cert = Initial::Cert(
        vote_plan_cert(
            &voters[0],
            chain_impl_mockchain::block::BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            &vote_plan,
        )
        .into(),
    );

    let config = Block0ConfigurationBuilder::default()
        .with_utxos(vec![
            voters[0].to_initial_fund(INITIAL_FUND_PER_WALLET_1),
            voters[1].to_initial_fund(INITIAL_FUND_PER_WALLET_2),
        ])
        .with_token(InitialToken {
            token_id: vote_plan.voting_token().clone().into(),
            policy: MintingPolicy::new().into(),
            to: vec![
                voters[0].to_initial_token(INITIAL_TOKEN_PER_WALLET_1),
                voters[1].to_initial_token(INITIAL_TOKEN_PER_WALLET_2),
            ],
        })
        .with_block0_consensus(ConsensusType::Bft)
        .with_kes_update_speed(KesUpdateSpeed::MAXIMUM)
        .with_treasury(INITIAL_TREASURY.into())
        .with_discrimination(Discrimination::Production)
        .with_committees(&[voters[0].to_committee_id()])
        .with_slot_duration(SLOT_DURATION.try_into().unwrap())
        .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
        .with_certs(vec![vote_plan_cert]);

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .with_block0_config(config)
        .build()
        .start_node(temp_dir)
        .unwrap();
    let settings = &jormungandr.rest().settings().unwrap();
    let transaction_sender = FragmentSender::from_settings(
        settings,
        chain_impl_mockchain::block::BlockDate {
            epoch: 1,
            slot_id: 0,
        }
        .into(),
        FragmentSenderSetup::resend_3_times(),
    );

    let fragment_builder = FragmentBuilder::from_settings(
        settings,
        chain_impl_mockchain::block::BlockDate {
            epoch: 3,
            slot_id: 0,
        },
    );

    let params = ExplorerParams::new(
        VOTE_PLAN_QUERY_COMPLEXITY_LIMIT,
        VOTE_PLAN_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let rewards_before: u64 = jormungandr.rest().remaining_rewards().unwrap().into();

    //1. Voteplan
    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    //2. Voting
    assert!(vote_plan.can_vote(get_current_date(&mut jormungandr.rest()).into()));

    let first_voter_luigi_fragment =
        fragment_builder.private_vote_cast(&voters[0], &vote_plan, VOTE_FOR_LUIGI, &yes_choice);

    let second_voter_luigi_fragment =
        fragment_builder.private_vote_cast(&voters[1], &vote_plan, VOTE_FOR_LUIGI, &yes_choice);
    voters[1].confirm_transaction();

    let second_voter_mario_fragment =
        fragment_builder.private_vote_cast(&voters[1], &vote_plan, VOTE_FOR_MARIO, &no_choice);

    transaction_sender
        .send_fragment(&mut voters[0], first_voter_luigi_fragment, &jormungandr)
        .unwrap();

    vote_for_luigi.push((chainWallet::from(voters[0].clone()), yes_choice));

    transaction_sender
        .send_fragment(&mut voters[1], second_voter_luigi_fragment, &jormungandr)
        .unwrap();

    vote_for_luigi.push((chainWallet::from(voters[1].clone()), yes_choice));

    transaction_sender
        .send_fragment(&mut voters[1], second_voter_mario_fragment, &jormungandr)
        .unwrap();

    vote_for_mario.push((chainWallet::from(voters[1].clone()), no_choice));

    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_MARIO as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_mario,
    );

    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_ANTONIO as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_antonio,
    );

    proposal_votes.insert(
        vote_plan
            .proposals()
            .to_vec()
            .get(VOTE_FOR_LUIGI as usize)
            .unwrap()
            .external_id()
            .to_string(),
        vote_for_luigi,
    );

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    //3.Tally
    wait_for_date(vote_plan.committee_start().into(), jormungandr.rest());
    let transaction_sender =
        transaction_sender.set_valid_until(chain_impl_mockchain::block::BlockDate {
            epoch: 3,
            slot_id: 0,
        });

    let vote_plan_statuses = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    let decrypted_shares = private_vote_committee_data_manager
        .decrypt_tally(&vote_plan_statuses.into())
        .unwrap();

    let mempool_check = transaction_sender
        .send_private_vote_tally(&mut voters[0], &vote_plan, decrypted_shares, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mempool_check,
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );

    //4. Tally end
    wait_for_date(vote_plan.committee_end().into(), jormungandr.rest());

    let rewards_after: u64 = jormungandr.rest().remaining_rewards().unwrap().into();

    // We want to make sure that our small rewards increase is reflected in current rewards amount
    assert!(
        rewards_after == rewards_before + REWARD_INCREASE,
        "Vote was unsuccessful"
    );

    let query_response = explorer
        .vote_plan(vote_plan.to_id().to_string())
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let vote_plan_transaction = query_response.data.unwrap().vote_plan;
    let vote_plan_status = jormungandr
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    ExplorerVerifier::assert_vote_plan_by_id(
        vote_plan_transaction,
        vote_plan_status,
        proposal_votes.clone(),
    );
}

#[test]
pub fn explorer_all_vote_plans_public_flow_test() {
    let temp_dir = TempDir::new().unwrap();
    let alice = Wallet::default();
    let bob = Wallet::default();
    let vote_plans_limit = 10;
    let mut voters = vec![alice, bob];
    let proposals = vec![VOTE_FOR_MARIO, VOTE_FOR_LUIGI, VOTE_FOR_ANTONIO];
    let proposal_count = proposals.len();
    let yes_choice = Choice::new(1);
    let no_choice = Choice::new(0);
    let mut vote_for_mario: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_luigi: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_antonio: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_plans_proposal_votes: HashMap<
        String,
        HashMap<String, Vec<(chainWallet, Choice)>>,
    > = HashMap::new();
    let vote_plans_count = 3;

    let vote_plans: Vec<VotePlan> = iter::from_fn(|| {
        Some(
            VotePlanBuilder::new()
                .proposals_count(proposal_count)
                .vote_start(BlockDate::from_epoch_slot_id(0, 0))
                .tally_start(BlockDate::from_epoch_slot_id(1, 0))
                .tally_end(BlockDate::from_epoch_slot_id(1, 10))
                .public()
                .build(),
        )
    })
    .take(vote_plans_count)
    .collect();

    let mut vote_plans_cert = Vec::new();

    for vote_plan in &vote_plans {
        let vote_plan_cert = Initial::Cert(
            vote_plan_cert(
                &voters[0],
                BlockDate {
                    epoch: 1,
                    slot_id: 0,
                },
                vote_plan,
            )
            .into(),
        );
        vote_plans_cert.push(vote_plan_cert);
    }

    let jormungandr = SingleNodeTestBootstrapper::default()
        .with_block0_config(
            Block0ConfigurationBuilder::default()
                .with_utxos(vec![
                    voters[0].to_initial_fund(INITIAL_FUND_PER_WALLET_1),
                    voters[1].to_initial_fund(INITIAL_FUND_PER_WALLET_2),
                ])
                .with_token(InitialToken {
                    token_id: vote_plans.first().unwrap().voting_token().clone().into(),
                    policy: MintingPolicy::new().into(),
                    to: vec![
                        voters[0].to_initial_token(INITIAL_TOKEN_PER_WALLET_1),
                        voters[1].to_initial_token(INITIAL_TOKEN_PER_WALLET_2),
                    ],
                })
                .with_committees(&[voters[0].to_committee_id()])
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_certs(vote_plans_cert)
                .with_treasury(INITIAL_TREASURY.into()),
        )
        .as_bft_leader()
        .build()
        .start_node(temp_dir)
        .unwrap();

    let transaction_sender = FragmentSender::try_from_with_setup(
        &jormungandr,
        BlockDate {
            epoch: 3,
            slot_id: 0,
        },
        FragmentSenderSetup::resend_3_times(),
    )
    .unwrap();

    let params = ExplorerParams::new(
        VOTE_PLAN_QUERY_COMPLEXITY_LIMIT,
        VOTE_PLAN_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();

    let explorer = explorer_process.client();

    // 1.Vote plan started
    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    assert!(vote_plans
        .first()
        .unwrap()
        .can_vote(get_current_date(&mut jormungandr.rest()).into()));

    //2. Voting
    for vote_plan in &vote_plans {
        transaction_sender
            .send_vote_cast(
                &mut voters[0],
                vote_plan,
                VOTE_FOR_ANTONIO,
                &no_choice,
                &jormungandr,
            )
            .unwrap();

        vote_for_antonio.push((chainWallet::from(voters[0].clone()), no_choice));

        transaction_sender
            .send_vote_cast(
                &mut voters[1],
                vote_plan,
                VOTE_FOR_ANTONIO,
                &yes_choice,
                &jormungandr,
            )
            .unwrap();

        vote_for_antonio.push((chainWallet::from(voters[1].clone()), yes_choice));

        transaction_sender
            .send_vote_cast(
                &mut voters[0],
                vote_plan,
                VOTE_FOR_MARIO,
                &no_choice,
                &jormungandr,
            )
            .unwrap();

        vote_for_mario.push((chainWallet::from(voters[0].clone()), no_choice));

        transaction_sender
            .send_vote_cast(
                &mut voters[1],
                vote_plan,
                VOTE_FOR_LUIGI,
                &no_choice,
                &jormungandr,
            )
            .unwrap();

        vote_for_luigi.push((chainWallet::from(voters[1].clone()), no_choice));

        let mut proposal_votes = HashMap::new();

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_MARIO as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_mario.clone(),
        );

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_ANTONIO as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_antonio.clone(),
        );

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_LUIGI as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_luigi.clone(),
        );

        vote_plans_proposal_votes.insert(vote_plan.to_id().to_string(), proposal_votes);
    }

    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    wait_for_date(
        vote_plans.first().unwrap().vote_end().into(),
        jormungandr.rest(),
    );

    //3.Start talling
    let mut mempool_check = Vec::new();
    for vote_plan in &vote_plans {
        mempool_check.push(
            transaction_sender
                .send_public_vote_tally(&mut voters[0], vote_plan, &jormungandr)
                .unwrap(),
        );
    }

    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mempool_check,
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    wait_for_date(
        vote_plans.first().unwrap().committee_end().into(),
        jormungandr.rest(),
    );

    //4. End talling
    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );
}

#[ignore]
#[test]
pub fn explorer_all_vote_plans_private_flow_test() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let yes_choice = Choice::new(1);
    let no_choice = Choice::new(2);
    let threshold = 1;
    let mut rng = OsRng;
    let alice = Wallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
    let bob = Wallet::new_account_with_discrimination(&mut rng, Discrimination::Production);
    let vote_plans_limit = 10;
    let mut voters = vec![alice, bob];
    let proposals = vec![VOTE_FOR_MARIO, VOTE_FOR_LUIGI, VOTE_FOR_ANTONIO];
    let proposal_count = proposals.len();
    let private_vote_committee_data_manager =
        CommitteeDataManager::private(&mut OsRng, vec![(voters[0].account_id())], threshold);
    let mut vote_for_mario: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_for_luigi: Vec<(chainWallet, Choice)> = Vec::new();
    let vote_for_antonio: Vec<(chainWallet, Choice)> = Vec::new();
    let mut vote_plans_proposal_votes: HashMap<
        String,
        HashMap<String, Vec<(chainWallet, Choice)>>,
    > = HashMap::new();
    let vote_plans_count = 3;

    let vote_plans: Vec<VotePlan> = iter::from_fn(|| {
        Some(
            VotePlanBuilder::new()
                .proposals_count(proposal_count)
                .vote_start(BlockDate::from_epoch_slot_id(0, 0))
                .tally_start(BlockDate::from_epoch_slot_id(1, 0))
                .tally_end(BlockDate::from_epoch_slot_id(1, 10))
                .private()
                .member_public_keys(private_vote_committee_data_manager.member_public_keys())
                .options_size(3)
                .build(),
        )
    })
    .take(vote_plans_count)
    .collect();

    let mut vote_plans_cert = Vec::new();

    for vote_plan in &vote_plans {
        let vote_plan_cert = Initial::Cert(
            vote_plan_cert(
                &voters[0],
                chain_impl_mockchain::block::BlockDate {
                    epoch: 1,
                    slot_id: 0,
                },
                vote_plan,
            )
            .into(),
        );
        vote_plans_cert.push(vote_plan_cert);
    }

    let jormungandr = SingleNodeTestBootstrapper::default()
        .with_block0_config(
            Block0ConfigurationBuilder::default()
                .with_utxos(vec![
                    voters[0].to_initial_fund(INITIAL_FUND_PER_WALLET_1),
                    voters[1].to_initial_fund(INITIAL_FUND_PER_WALLET_2),
                ])
                .with_token(InitialToken {
                    token_id: vote_plans.first().unwrap().voting_token().clone().into(),
                    policy: MintingPolicy::new().into(),
                    to: vec![
                        voters[0].to_initial_token(INITIAL_FUND_PER_WALLET_1),
                        voters[1].to_initial_token(INITIAL_FUND_PER_WALLET_2),
                    ],
                })
                .with_block0_consensus(ConsensusType::Bft)
                .with_kes_update_speed(KesUpdateSpeed::MAXIMUM)
                .with_treasury(INITIAL_TREASURY.into())
                .with_discrimination(Discrimination::Production)
                .with_committees(&[voters[0].to_committee_id()])
                .with_slot_duration(SLOT_DURATION.try_into().unwrap())
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_certs(vote_plans_cert),
        )
        .as_bft_leader()
        .build()
        .start_node(temp_dir)
        .unwrap();

    let transaction_sender = FragmentSender::try_from_with_setup(
        &jormungandr,
        BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        FragmentSenderSetup::resend_3_times(),
    )
    .unwrap();

    let fragment_builder = FragmentBuilder::try_from_with_setup(
        &jormungandr,
        BlockDate {
            epoch: 3,
            slot_id: 0,
        },
    )
    .unwrap();

    let params = ExplorerParams::new(
        VOTE_PLAN_QUERY_COMPLEXITY_LIMIT,
        VOTE_PLAN_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    //1. Voteplan
    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    //2. Voting
    assert!(vote_plans
        .first()
        .unwrap()
        .can_vote(get_current_date(&mut jormungandr.rest()).into()));

    for vote_plan in &vote_plans {
        let first_voter_luigi_fragment =
            fragment_builder.private_vote_cast(&voters[0], vote_plan, VOTE_FOR_LUIGI, &yes_choice);

        let second_voter_luigi_fragment =
            fragment_builder.private_vote_cast(&voters[1], vote_plan, VOTE_FOR_LUIGI, &yes_choice);
        voters[1].confirm_transaction();

        let second_voter_mario_fragment =
            fragment_builder.private_vote_cast(&voters[1], vote_plan, VOTE_FOR_MARIO, &no_choice);
        voters[1].decrement_counter();

        transaction_sender
            .send_fragment(&mut voters[0], first_voter_luigi_fragment, &jormungandr)
            .unwrap();

        vote_for_luigi.push((chainWallet::from(voters[0].clone()), yes_choice));

        transaction_sender
            .send_fragment(&mut voters[1], second_voter_luigi_fragment, &jormungandr)
            .unwrap();

        vote_for_luigi.push((chainWallet::from(voters[1].clone()), yes_choice));

        transaction_sender
            .send_fragment(&mut voters[1], second_voter_mario_fragment, &jormungandr)
            .unwrap();

        vote_for_mario.push((chainWallet::from(voters[1].clone()), no_choice));

        let mut proposal_votes = HashMap::new();

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_MARIO as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_mario.clone(),
        );

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_ANTONIO as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_antonio.clone(),
        );

        proposal_votes.insert(
            vote_plan
                .proposals()
                .to_vec()
                .get(VOTE_FOR_LUIGI as usize)
                .unwrap()
                .external_id()
                .to_string(),
            vote_for_luigi.clone(),
        );

        vote_plans_proposal_votes.insert(vote_plan.to_id().to_string(), proposal_votes);
    }

    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    //3.Tally
    wait_for_date(
        vote_plans.first().unwrap().committee_start().into(),
        jormungandr.rest(),
    );
    let transaction_sender =
        transaction_sender.set_valid_until(chain_impl_mockchain::block::BlockDate {
            epoch: 3,
            slot_id: 0,
        });

    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();

    let mut mempool_check = Vec::new();
    for vote_plan_status in vote_plan_statuses {
        let decrypted_shares = private_vote_committee_data_manager
            .decrypt_tally(&vote_plan_status.clone().into())
            .unwrap();

        for vote_plan in &vote_plans {
            if vote_plan.to_id().to_string() == vote_plan_status.id.to_string() {
                mempool_check.push(
                    transaction_sender
                        .send_private_vote_tally(
                            &mut voters[0],
                            vote_plan,
                            decrypted_shares.clone(),
                            &jormungandr,
                        )
                        .unwrap(),
                );
            }
        }
    }

    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mempool_check,
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );

    //4. Tally end
    wait_for_date(
        vote_plans.first().unwrap().committee_end().into(),
        jormungandr.rest(),
    );

    let query_response = explorer
        .vote_plans(vote_plans_limit)
        .expect("vote plan transaction not found");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let all_vote_plans_response = query_response.data.unwrap().tip.all_vote_plans;
    let all_vote_plans = all_vote_plans_response.edges;
    let vote_plan_statuses = jormungandr.rest().vote_plan_statuses().unwrap();
    assert_eq!(
        all_vote_plans_response.total_count,
        vote_plan_statuses.len() as i64
    );

    ExplorerVerifier::assert_all_vote_plans(
        all_vote_plans,
        vote_plan_statuses,
        vote_plans_proposal_votes.clone(),
    );
}
