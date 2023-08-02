use crate::startup::{self, SingleNodeTestBootstrapper};
use assert_fs::{prelude::*, TempDir};
use jormungandr_automation::jormungandr::{
    explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier},
    Block0ConfigurationBuilder, NodeConfigBuilder,
};
use std::{iter, time::Duration};
use thor::{
    Block0ConfigurationBuilderExtension, FragmentSender, FragmentVerifier, StakePool, Wallet,
};

const STAKE_POOL_QUERY_COMPLEXITY_LIMIT: u64 = 100;
const STAKE_POOL_QUERY_DEPTH_LIMIT: u64 = 30;

#[test]
pub fn explorer_not_existing_stake_pool_test() {
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
        .stake_pool(stake_pool_id, stake_pool_block_count)
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

#[should_panic] //BUG NPG-3915
#[test]
pub fn explorer_stake_pool_test() {
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

    let settings = jormungandr.rest().settings().unwrap();
    let fragment_sender = FragmentSender::from(&settings);

    let params = ExplorerParams::new(
        STAKE_POOL_QUERY_COMPLEXITY_LIMIT,
        STAKE_POOL_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let initial_stake_pool = stake_pools.get(0).unwrap();

    let mut stake_pool_update = initial_stake_pool.clone();
    let stake_pool_info = stake_pool_update.info_mut();
    stake_pool_info.owners = vec![stake_pool_owner.public_key()];

    let mem_check = fragment_sender
        .send_pool_update(
            &mut faucet,
            initial_stake_pool,
            &stake_pool_update,
            &jormungandr,
        )
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(Duration::from_secs(2), mem_check, &jormungandr)
        .unwrap();

    let query_response = explorer
        .stake_pool(initial_stake_pool.id().to_string(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    ExplorerVerifier::assert_stake_pool(stake_pool_update.inner(), &explorer_stake_pool, None);

    let stake_pool_id = stake_pool.id().to_string();

    let mem_check = fragment_sender
        .send_pool_registration(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .expect("Error while sending registration certificate for stake pool owner");

    FragmentVerifier::wait_and_verify_is_in_block(Duration::from_secs(2), mem_check, &jormungandr)
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

    let mem_check = fragment_sender
        .send_owner_delegation(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(Duration::from_secs(2), mem_check, &jormungandr)
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

    //FIXME add delegation check when NPG-2247 has been fixed
    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    let mem_check = fragment_sender
        .send_full_delegation(&mut full_delegator, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(Duration::from_secs(2), mem_check, &jormungandr)
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

    //FIXME add delegation check when NPG-2247 has been fixed
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
        .stake_pool(stake_pool_id, stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    //FIXME add delegation check when NPG-2247 has been fixed
    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool, None);

    fragment_sender
        .send_pool_retire(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(Duration::from_secs(2), mem_check, &jormungandr)
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
    let _faucet = thor::Wallet::default();
    let stake_pool_owner = thor::Wallet::default();
    let _stake_pool = StakePool::new(&stake_pool_owner);
    let _full_delegator = thor::Wallet::default();
    let _split_delegator = thor::Wallet::default();
    let stake_pool_owners: Vec<Wallet> = iter::from_fn(|| Some(thor::Wallet::default()))
        .take(6)
        .collect();
    let stake_pool_count = 10;

    let (jormungandr, stake_pools) = startup::start_stake_pool(
        &stake_pool_owners,
        &[stake_pool_owners[0].clone()],
        Block0ConfigurationBuilder::default(),
        NodeConfigBuilder::default().with_storage(temp_dir.child("storage").to_path_buf()),
    )
    .unwrap();

    let params = ExplorerParams::new(
        STAKE_POOL_QUERY_COMPLEXITY_LIMIT,
        STAKE_POOL_QUERY_DEPTH_LIMIT,
        None,
    );
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let query_response = explorer
        .stake_pools(stake_pool_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pools = query_response.data.unwrap().tip.all_stake_pools;
    let stake_pools_inner = stake_pools.iter().map(|x| x.inner().to_owned()).collect();

    ExplorerVerifier::assert_all_stake_pools(stake_pools_inner, explorer_stake_pools);
}
