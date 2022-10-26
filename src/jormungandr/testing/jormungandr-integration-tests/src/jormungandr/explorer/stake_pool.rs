use std::{time::Duration, num::NonZeroU64};

use crate::startup::SingleNodeTestBootstrapper;
use assert_fs::TempDir;
use chain_crypto::{testing, PublicKey, SumEd25519_12};
use chain_impl_mockchain::{
    testing::{StakePoolBuilder, TestGen},
    transaction::AccountIdentifier, value::Value, header::BlockDate,
};
use jormungandr_automation::{jormungandr::{
    explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier},
    Block0ConfigurationBuilder,
}, testing::time::wait_for_date};
use thor::{Block0ConfigurationBuilderExtension, FragmentSender, StakePool, FragmentVerifier};

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
    let temp_dir = TempDir::new().unwrap();
    let mut stake_pool_owner = thor::Wallet::default();
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

    fragment_sender
        .send_pool_registration(&mut stake_pool_owner, &stake_pool, &jormungandr)
        .expect("Error while sending registration certificate for first stake pool owner");

    let query_response = explorer
        .stake_pool(stake_pool_id.clone(), stake_pool_block_count)
        .expect("Non existing stake pool");

    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    ExplorerVerifier::assert_stake_pool(stake_pool.inner(), &explorer_stake_pool);

    let mut stake_pool_update = stake_pool.clone();
    let mut stake_pool_info = stake_pool_update.info_mut();
    stake_pool_info.reward_account = Some(AccountIdentifier::Single(TestGen::identifier()));


        let  pk_kes: PublicKey<SumEd25519_12> =
                testing::static_secret_key::<SumEd25519_12>().to_public();


     stake_pool_info.keys.kes_public_key = pk_kes;
     stake_pool_info.


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

        wait_for_date(
            BlockDate {
                epoch: 1,
                slot_id: 0,
            }
            .into(),
            jormungandr.rest(),
        );

    let query_response = explorer
        .stake_pool(stake_pool_update.id().to_string(), stake_pool_block_count)
        .expect("Non existing stake pool");



    assert!(
        query_response.errors.is_none(),
        "{:?}",
        query_response.errors.unwrap()
    );

    let explorer_stake_pool = query_response.data.unwrap().stake_pool;

    ExplorerVerifier::assert_stake_pool(stake_pool_update.inner(), &explorer_stake_pool);
    //fragment_sender.send_pool_retire(from, to, via)
    //fragment_sender.send_owner_delegation(from, to, via)
}
