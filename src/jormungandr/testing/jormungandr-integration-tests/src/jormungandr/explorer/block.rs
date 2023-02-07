use std::time::Duration;

use crate::startup::SingleNodeTestBootstrapper;
use assert_fs::TempDir;
use chain_core::{
    packer::Codec,
    property::{Deserialize, FromStr},
};
use chain_crypto::Ed25519;
use chain_impl_mockchain::{
    block::Block,
    chaintypes::ConsensusType,
    fee::LinearFee,
    tokens::{identifier::TokenIdentifier, minting_policy::MintingPolicy},
};
use jormungandr_automation::{
    jcli::JCli,
    jormungandr::{
        explorer::{configuration::ExplorerParams, verifiers::ExplorerVerifier},
        Block0ConfigurationBuilder, MemPoolCheck, NodeConfigBuilder,
    },
    testing::{block0::Block0ConfigurationExtension, keys::create_new_key_pair, time},
};
use jormungandr_lib::interfaces::{
    ActiveSlotCoefficient, BlockDate, FragmentStatus, InitialToken, Mempool,
};
use mjolnir::generators::FragmentGenerator;
use thor::{
    Block0ConfigurationBuilderExtension, FragmentSender, FragmentSenderSetup, FragmentVerifier,
};

const BLOCK_QUERY_COMPLEXITY_LIMIT: u64 = 150;
const BLOCK_QUERY_DEPTH_LIMIT: u64 = 30;
const SLOTS_PER_EPOCH: u32 = 20;
const SLOT_DURATION: u8 = 2;

#[test]
pub fn explorer_block_test() {
    let temp_dir = TempDir::new().unwrap();
    let receiver = thor::Wallet::default();
    let sender = thor::Wallet::default();
    let bft_secret = create_new_key_pair::<Ed25519>();
    let jcli: JCli = Default::default();

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .with_block0_config(
            Block0ConfigurationBuilder::default()
                .with_consensus_leaders_ids(vec![bft_secret.identifier().into()])
                .with_wallets_having_some_values(vec![&sender, &receiver])
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_block_content_max_size(100000.into())
                .with_slot_duration(SLOT_DURATION.try_into().unwrap())
                .with_token(InitialToken {
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

    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();
    let settings = jormungandr.rest().settings().unwrap();

    let fragment_sender =
        FragmentSender::from_settings_with_setup(&settings, FragmentSenderSetup::resend_3_times());

    let time_era = jormungandr.time_era();

    let mut fragment_generator = FragmentGenerator::new(
        sender,
        receiver,
        Some(bft_secret),
        jormungandr.to_remote(),
        time_era.slots_per_epoch(),
        2,
        2,
        2,
        2,
        fragment_sender.clone(),
    );

    fragment_generator.prepare(BlockDate::new(1, 0));

    time::wait_for_epoch(2, jormungandr.rest());

    let mem_check = fragment_generator.send_random().unwrap();
    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::from_secs(2),
        mem_check.clone(),
        &jormungandr,
    )
    .unwrap();
    let fragments_log = jcli.rest().v0().message().logs(jormungandr.rest_uri());
    let fragment_log = fragments_log
        .iter()
        .find(|x| *x.fragment_id().to_string() == mem_check.fragment_id().to_string())
        .unwrap();

    let fragment_block_id =
        if let &FragmentStatus::InABlock { date: _, block } = fragment_log.status() {
            block
        } else {
            panic!("Fragment not in block")
        };

    let encoded_block = jcli
        .rest()
        .v0()
        .block()
        .get(fragment_block_id.to_string(), jormungandr.rest_uri());

    let bytes_block = hex::decode(encoded_block.trim()).unwrap();
    let reader = std::io::Cursor::new(&bytes_block);
    let decoded_block = Block::deserialize(&mut Codec::new(reader)).unwrap();

    let explorer = explorer_process.client();

    let explorer_block_response = explorer.block_by_id(fragment_block_id.to_string()).unwrap();

    assert!(
        explorer_block_response.errors.is_none(),
        "{:?}",
        explorer_block_response.errors.unwrap()
    );

    let explorer_block = explorer_block_response.data.unwrap().block;

    ExplorerVerifier::assert_block_by_id(decoded_block, explorer_block).unwrap();
}

#[test]
pub fn explorer_block0_test() {
    let temp_dir = TempDir::new().unwrap();
    let test_context = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .build();
    let jormungandr = test_context.start_node(temp_dir).unwrap();
    let block0_id = test_context.block0_config().to_block_hash().to_string();
    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    let explorer_block0_response = explorer.block_by_id(block0_id).unwrap();

    assert!(
        explorer_block0_response.errors.is_none(),
        "{:?}",
        explorer_block0_response.errors.unwrap()
    );

    let explorer_block0 = explorer_block0_response.data.unwrap().block;
    let block0 = test_context.block0_config().to_block();
    ExplorerVerifier::assert_block_by_id(block0, explorer_block0).unwrap();
}

#[test]
pub fn explorer_block_incorrect_id_test() {
    let temp_dir = TempDir::new().unwrap();
    let incorrect_block_ids = vec![
        (
            "e1049ea45726f0b1fc473af54f706546b3331765abf89ae9e6a8333e49621641aa",
            "invalid hash size",
        ),
        (
            "e1049ea45726f0b1fc473af54f706546b3331765abf89ae9e6a8333e49621641a",
            "invalid hex encoding",
        ),
        (
            "e1049ea45726f0b1fc473af54f706546b3331765abf89ae9e6a8333e49621641",
            "Couldn't find block in the explorer",
        ),
    ];

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_bft_leader()
        .build()
        .start_node(temp_dir)
        .unwrap();

    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();
    let explorer = explorer_process.client();

    for (incorrect_block_id, error_message) in incorrect_block_ids {
        let response = explorer.block_by_id(incorrect_block_id.to_string());
        assert!(response.as_ref().unwrap().errors.is_some());
        assert!(response.as_ref().unwrap().data.is_none());
        assert!(response
            .unwrap()
            .errors
            .unwrap()
            .first()
            .unwrap()
            .message
            .contains(error_message));
    }
}

#[test]
pub fn explorer_last_block_test() {
    let jcli: JCli = Default::default();
    let temp_dir = TempDir::new().unwrap();
    let sender = thor::Wallet::default();
    let receiver = thor::Wallet::default();
    let stake_pool = thor::StakePool::new(&sender);

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_genesis_praos_stake_pool(&stake_pool)
        .with_block0_config(
            Block0ConfigurationBuilder::minimal_setup()
                .with_wallets_having_some_values(vec![&sender, &receiver])
                .with_stake_pool_and_delegation(&stake_pool, vec![&sender])
                .with_block0_consensus(ConsensusType::GenesisPraos)
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_block_content_max_size(100000.into())
                .with_consensus_genesis_praos_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
                .with_slot_duration(SLOT_DURATION.try_into().unwrap())
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

    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();

    let fragment_sender = FragmentSender::from_settings_with_setup(
        &jormungandr.rest().settings().unwrap(),
        FragmentSenderSetup::no_verify(),
    );

    let time_era = jormungandr.time_era();

    let mut fragment_generator = FragmentGenerator::new(
        sender,
        receiver,
        None,
        jormungandr.to_remote(),
        time_era.slots_per_epoch(),
        2,
        2,
        2,
        0,
        fragment_sender,
    );

    fragment_generator.prepare(BlockDate::new(1, 0));

    time::wait_for_epoch(2, jormungandr.rest());

    let mem_checks: Vec<MemPoolCheck> = fragment_generator.send_all().unwrap();

    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mem_checks,
        &jormungandr,
    )
    .unwrap();

    time::wait_for_epoch(3, jormungandr.rest());

    assert!(explorer_process.wait_to_be_up(2, 10));
    let explorer = explorer_process.client();
    let explorer_block_response = explorer.last_block();
    let explorer_last_block = explorer_block_response.unwrap();

    let encoded_block = jcli
        .rest()
        .v0()
        .block()
        .get(&explorer_last_block.block().id, jormungandr.rest_uri());

    let bytes_block = hex::decode(encoded_block.trim()).unwrap();
    let reader = std::io::Cursor::new(&bytes_block);
    let decoded_block = Block::deserialize(&mut Codec::new(reader)).unwrap();

    assert_eq!(
        explorer_last_block.block_date(),
        decoded_block.header().block_date().into()
    );

    ExplorerVerifier::assert_last_block(decoded_block, explorer_last_block.block()).unwrap();
}

#[should_panic] //NPG-3517
#[test]
pub fn explorer_all_blocks_test() {
    let jcli: JCli = Default::default();
    let temp_dir = TempDir::new().unwrap();
    let sender = thor::Wallet::default();
    let receiver = thor::Wallet::default();
    let stake_pool = thor::StakePool::new(&sender);
    let max_blocks_number = 100;

    let test_context = SingleNodeTestBootstrapper::default()
        .as_genesis_praos_stake_pool(&stake_pool)
        .with_block0_config(
            Block0ConfigurationBuilder::minimal_setup()
                .with_wallets_having_some_values(vec![&sender, &receiver])
                .with_stake_pool_and_delegation(&stake_pool, vec![&sender])
                .with_block0_consensus(ConsensusType::GenesisPraos)
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_block_content_max_size(100000.into())
                .with_consensus_genesis_praos_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
                .with_slot_duration(SLOT_DURATION.try_into().unwrap())
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
        .build();

    let jormungandr = test_context.start_node(temp_dir).unwrap();
    let block0_id = test_context.block0_config().to_block_hash();

    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();

    let fragment_sender = FragmentSender::from_settings_with_setup(
        &jormungandr.rest().settings().unwrap(),
        FragmentSenderSetup::no_verify(),
    );

    let time_era = jormungandr.time_era();

    let mut fragment_generator = FragmentGenerator::new(
        sender,
        receiver,
        None,
        jormungandr.to_remote(),
        time_era.slots_per_epoch(),
        2,
        2,
        2,
        0,
        fragment_sender,
    );

    fragment_generator.prepare(BlockDate::new(1, 0));

    time::wait_for_epoch(2, jormungandr.rest());

    let mem_checks: Vec<MemPoolCheck> = fragment_generator.send_all().unwrap();

    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mem_checks,
        &jormungandr,
    )
    .unwrap();

    time::wait_for_epoch(3, jormungandr.rest());

    assert!(explorer_process.wait_to_be_up(2, 10));

    let explorer = explorer_process.client();
    let explorer_block_response = explorer.blocks(max_blocks_number).unwrap();

    assert!(
        explorer_block_response.errors.is_none(),
        "{:?}",
        explorer_block_response.errors.unwrap()
    );

    let explorer_blocks_data = explorer_block_response.data.unwrap();
    let explorer_blocks = explorer_blocks_data.tip.blocks.edges;

    let mut block_ids = jcli.rest().v0().block().next(
        block0_id.to_string(),
        (explorer_blocks.len() - 1) as u32,
        jormungandr.rest_uri(),
    );

    block_ids.insert(0, block0_id);
    assert_eq!(explorer_blocks.len(), block_ids.len());

    for (n, explorer_block) in explorer_blocks.iter().enumerate() {
        let encoded_block = jcli
            .rest()
            .v0()
            .block()
            .get(block_ids[n].to_string(), jormungandr.rest_uri());
        let decoded_block = ExplorerVerifier::decode_block(encoded_block);
        ExplorerVerifier::assert_all_blocks(decoded_block, &explorer_block.node).unwrap()
    }
}

#[test]
pub fn explorer_block_by_chain_length_test() {
    let temp_dir = TempDir::new().unwrap();
    let jcli: JCli = Default::default();
    let sender = thor::Wallet::default();
    let receiver = thor::Wallet::default();
    let stake_pool = thor::StakePool::new(&sender);

    let jormungandr = SingleNodeTestBootstrapper::default()
        .as_genesis_praos_stake_pool(&stake_pool)
        .with_block0_config(
            Block0ConfigurationBuilder::minimal_setup()
                .with_wallets_having_some_values(vec![&sender, &receiver])
                .with_stake_pool_and_delegation(&stake_pool, vec![&sender])
                .with_block0_consensus(ConsensusType::GenesisPraos)
                .with_slots_per_epoch(SLOTS_PER_EPOCH.try_into().unwrap())
                .with_block_content_max_size(100000.into())
                .with_consensus_genesis_praos_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
                .with_slot_duration(SLOT_DURATION.try_into().unwrap())
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

    let params = ExplorerParams::new(BLOCK_QUERY_COMPLEXITY_LIMIT, BLOCK_QUERY_DEPTH_LIMIT, None);
    let explorer_process = jormungandr.explorer(params).unwrap();

    let fragment_sender = FragmentSender::from_settings_with_setup(
        &jormungandr.rest().settings().unwrap(),
        FragmentSenderSetup::no_verify(),
    );

    let time_era = jormungandr.time_era();

    let mut fragment_generator = FragmentGenerator::new(
        sender,
        receiver,
        None,
        jormungandr.to_remote(),
        time_era.slots_per_epoch(),
        2,
        2,
        2,
        0,
        fragment_sender,
    );

    fragment_generator.prepare(BlockDate::new(1, 0));

    time::wait_for_epoch(2, jormungandr.rest());

    let mem_checks: Vec<MemPoolCheck> = fragment_generator.send_all().unwrap();

    FragmentVerifier::wait_and_verify_all_are_in_block(
        Duration::from_secs(2),
        mem_checks.clone(),
        &jormungandr,
    )
    .unwrap();

    let fragments_log = jcli.rest().v0().message().logs(jormungandr.rest_uri());
    let fragment_log = fragments_log
        .iter()
        .find(|x| {
            *x.fragment_id().to_string() == mem_checks.last().unwrap().fragment_id().to_string()
        })
        .unwrap();

    let fragment_block_id =
        if let &FragmentStatus::InABlock { date: _, block } = fragment_log.status() {
            block
        } else {
            panic!("Fragment not in block")
        };

    let encoded_block = jcli
        .rest()
        .v0()
        .block()
        .get(fragment_block_id.to_string(), jormungandr.rest_uri());

    let decoded_block = ExplorerVerifier::decode_block(encoded_block);

    time::wait_for_epoch(3, jormungandr.rest());

    explorer_process.wait_to_be_up(2, 20);
    let explorer = explorer_process.client();

    let explorer_block_response = explorer
        .blocks_at_chain_length(decoded_block.header().chain_length().into())
        .unwrap();

    assert!(
        explorer_block_response.errors.is_none(),
        "{:?}",
        explorer_block_response.errors.unwrap()
    );

    let explorer_blocks_data = explorer_block_response.data.unwrap();
    let explorer_block = explorer_blocks_data.blocks_by_chain_length.first().unwrap();

    ExplorerVerifier::assert_block_by_chain_length(decoded_block, explorer_block).unwrap();
}
