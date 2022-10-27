use crate::startup::{self, create_new_leader_key, SingleNodeTestBootstrapper};
use assert_fs::{prelude::*, TempDir};
use chain_addr::Discrimination;
use chain_core::property::BlockDate as propertyBlockDate;
use chain_crypto::{testing, PublicKey, SumEd25519_12};
use chain_impl_mockchain::{
    account::DelegationType,
    block::BlockDate,
    certificate::{VoteAction, VotePlan},
    chaintypes::ConsensusType,
    fee::LinearFee,
    ledger::governance::TreasuryGovernanceAction,
    testing::{data::Wallet as chainWallet, StakePoolBuilder, TestGen},
    tokens::{identifier::TokenIdentifier, minting_policy::MintingPolicy},
    transaction::AccountIdentifier,
    value::Value,
    vote::Choice,
};
use jormungandr_automation::{
    jcli::JCli,
    jormungandr::{
        explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier},
        Block0ConfigurationBuilder, MemPoolCheck, NodeConfigBuilder,
    },
    testing::{
        time::{self, get_current_date, wait_for_date},
        VotePlanBuilder,
    },
};
use jormungandr_lib::interfaces::{
    ActiveSlotCoefficient, BlockDate as libBlockDate, Initial, InitialToken, KesUpdateSpeed,
    Mempool,
};
use mjolnir::generators::FragmentGenerator;
use rand_core::OsRng;
use std::{collections::HashMap, iter, num::NonZeroU64, time::Duration};
use thor::{
    vote_plan_cert, Block0ConfigurationBuilderExtension, CommitteeDataManager, FragmentBuilder,
    FragmentSender, FragmentSenderSetup, FragmentVerifier, StakePool, Wallet,
};

const STAKE_POOL_QUERY_COMPLEXITY_LIMIT: u64 = 100;
const STAKE_POOL_QUERY_DEPTH_LIMIT: u64 = 30;

#[test]
pub fn explorer_not_existing_stake_pool_test() {
    let jcli: JCli = Default::default();
    let temp_dir = TempDir::new().unwrap();
    let stake_pool_owner = thor::Wallet::default();
    let stake_pool = StakePool::new(&stake_pool_owner);
    let stake_pool_block_count = 10;

    let jormungandr = SingleNodeTestBootstrapper::default()
        .with_block0_config(
            Block0ConfigurationBuilder::default().with_wallet(&stake_pool_owner, 1_000_000.into()),
        )
        .as_bft_leader()
        .build()
        .start_node(temp_dir)
        .unwrap();

    let params = ExplorerParams::new(
        STAKE_POOL_QUERY_COMPLEXITY_LIMIT,
        STAKE_POOL_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();
    let stake_pool_id = stake_pool.id().to_string();

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

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
pub fn explorer_stake_pool_test() {
    let jcli: JCli = Default::default();
    let temp_dir = TempDir::new().unwrap();
    let mut faucet = thor::Wallet::default();
    let mut stake_pool_owner = thor::Wallet::default();
    let stake_pool = StakePool::new(&stake_pool_owner);
    let mut full_delegator = thor::Wallet::default();
    let mut split_delegator = thor::Wallet::default();
    let stake_pool_block_count = 10;

    let (jormungandr, stake_pools) = startup::start_stake_pool(
        &[faucet.clone()],
        &[full_delegator.clone(), split_delegator.clone()],
        Block0ConfigurationBuilder::default().with_wallet(&stake_pool_owner, 1_000_000.into()),
        NodeConfigBuilder::default().with_storage(temp_dir.child("storage").to_path_buf()),
    )
    .unwrap();

    let initial_stake_pool = stake_pools.get(0).unwrap();

    let settings = jormungandr.rest().settings().unwrap();
    let fragment_sender = FragmentSender::from(&settings);

    let params = ExplorerParams::new(
        STAKE_POOL_QUERY_COMPLEXITY_LIMIT,
        STAKE_POOL_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();
    let stake_pool_id = stake_pool.id().to_string();

    let mem_check = fragment_sender
        .send_pool_registration(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .expect("Error while sending registration certificate for stake pool owner");

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    let mut stake_pool_update = stake_pool.clone();
    let mut stake_pool_info = stake_pool_update.info_mut();

    stake_pool_info.reward_account = Some(AccountIdentifier::Single(TestGen::identifier()));
    stake_pool_info.keys.kes_public_key = testing::static_secret_key::<SumEd25519_12>().to_public();

    println!("OWNERS {:?}", faucet.public_key());
    stake_pool_info.owners = vec![faucet.public_key()];

    let mem_check = fragment_sender
        .send_pool_update(
            &mut stake_pool_owner,
            &stake_pool,
            &stake_pool_update,
            &jormungandr,
        )
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_update.id().to_string(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    use std::{thread, time};
    thread::sleep(time::Duration::from_secs(600));

    //ExplorerVerifier::assert_stake_pool(stake_pool_update.inner(), &explorer_stake_pool, None);

    let mem_check = fragment_sender
        .send_owner_delegation(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;
    //PANIC
    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    let mem_check = fragment_sender
        .send_full_delegation(&mut full_delegator, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;
    //PANIC
    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    let mem_check = fragment_sender
        .send_split_delegation(
            &mut split_delegator,
            &[(initial_stake_pool, 1u8), (&stake_pool, 1u8)],
            &jormungandr,
        )
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;
    //PANIC
    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    let mut stake_pool_update = stake_pool.clone();
    let mut stake_pool_info = stake_pool_update.info_mut();

    stake_pool_info.reward_account = Some(AccountIdentifier::Single(TestGen::identifier()));
    stake_pool_info.keys.kes_public_key = testing::static_secret_key::<SumEd25519_12>().to_public();

    let mem_check = fragment_sender
        .send_pool_update(
            &mut stake_pool_owner,
            &stake_pool,
            &stake_pool_update,
            &jormungandr,
        )
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool_update.id().to_string(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    //ExplorerVerifier::assert_stake_pool(stake_pool_update.inner(), &explorer_stake_pool, None);

    fragment_sender
        .send_pool_retire(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();

    let query_response = explorer
        .stake_pool(stake_pool.id().to_string(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);
}

#[test]
pub fn explorer_all_stake_pool_test() {
    let temp_dir = TempDir::new().unwrap();
    let jcli: JCli = Default::default();
    let sender = thor::Wallet::default();
    let receiver = thor::Wallet::default();
    let bft_leader = create_new_leader_key();
    let stake_pool_count = 100;

    let jormungandr = SingleNodeTestBootstrapper::default()
        .with_bft_secret(bft_leader.signing_key())
        .with_block0_config(
            Block0ConfigurationBuilder::default()
                .with_wallets_having_some_values(vec![&sender, &receiver])
                .with_block0_consensus(ConsensusType::Bft)
                .with_slots_per_epoch(20.try_into().unwrap())
                .with_block_content_max_size(100000.into())
                .with_consensus_leaders_ids(vec![bft_leader.identifier().into()])
                .with_consensus_genesis_praos_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
                .with_slot_duration(3.try_into().unwrap())
                .with_linear_fees(LinearFee::new(1, 1, 1))
                .with_token(InitialToken {
                    // FIXME: this works because I know it's the VotePlanBuilder's default, but
                    // probably should me more explicit.
                    token_id: TokenIdentifier::from_str(
                        "00000000000000000000000000000000000000000000000000000000.00000000",
                    )
                    .unwrap()
                    .into(),
                    policy: MintingPolicy::new().into(),
                    to: vec![sender.to_initial_token(1_000_000)],
                }),
        )
        .with_node_config(NodeConfigBuilder::default().with_mempool(Mempool {
            pool_max_entries: 1_000_000usize.into(),
            log_max_entries: 1_000_000usize.into(),
            persistent_log: None,
        }))
        .build()
        .start_node(temp_dir)
        .unwrap();

    let params = ExplorerParams::new(
        STAKE_POOL_QUERY_COMPLEXITY_LIMIT,
        STAKE_POOL_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let fragment_sender = FragmentSender::try_from_with_setup(
        &jormungandr,
        libBlockDate::new(1u32, 1u32).next_epoch().into(),
        FragmentSenderSetup::no_verify(),
    )
    .unwrap();

    let time_era = jormungandr.time_era();

    let mut fragment_generator = FragmentGenerator::new(
        sender,
        receiver,
        Some(bft_leader),
        jormungandr.to_remote(),
        time_era.slots_per_epoch(),
        10,
        2,
        2,
        2,
        fragment_sender,
    );

    fragment_generator.prepare(libBlockDate::new(1, 0));

    time::wait_for_epoch(2, jormungandr.rest());

    let mem_checks: Vec<MemPoolCheck> = fragment_generator.send_all().unwrap();
    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mem_checks.clone(),
        &jormungandr,
    )
    .unwrap();

    let stake_pools = jormungandr.rest().stake_pools();

    let query_response = explorer
        .stake_pools(stake_pool_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pools = query_response.data.unwrap().tip.all_stake_pools;

    //ExplorerVerifier::assert_all_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);
}
