use assert_fs::TempDir;
use jormungandr_automation::jormungandr::{Block0ConfigurationBuilder, explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier}};
use thor::{StakePool, FragmentSender, Block0ConfigurationBuilderExtension};

use crate::startup::SingleNodeTestBootstrapper;
const STAKE_POOL_QUERY_COMPLEXITY_LIMIT: u64 = 100;
const STAKE_POOL_QUERY_DEPTH_LIMIT: u64 = 30;

#[test]
pub fn explorer_stake_pool_test() {
    let temp_dir = TempDir::new().unwrap();
    let mut stake_pool_owner = thor::Wallet::default();
    let stake_pool = StakePool::new(&stake_pool_owner);
    let stake_pool_block_count = 10;

    let jormungandr = SingleNodeTestBootstrapper::default()
        .with_block0_config(
            Block0ConfigurationBuilder::default()
                .with_wallet(&stake_pool_owner, 1_000_000.into()),
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

    fragment_sender.send_pool_registration(
            &mut take_pool_owner,
            &stake_pool,
            &jormungandr,
        )
        .expect("Error while sending registration certificate for first stake pool owner");
        
    first_stake_pool.id().to_string()

    let query_response = explorer
        .stake_pool(id, stake_pool_block_count)
        .expect("Non existing stake pool ");

    assert!(query_response.errors.is_none(), "{:?}", query_response.errors.unwrap());

    //fragment_sender.send_pool_update(from, to, update_stake_pool, via)
    //fragment_sender.send_pool_retire(from, to, via)
    //fragment_sender.send_owner_delegation(from, to, via)

}