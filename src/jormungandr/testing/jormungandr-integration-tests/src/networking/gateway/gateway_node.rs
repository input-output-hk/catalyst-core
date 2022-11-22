use crate::{build_network, jormungandr::grpc::setup::client, networking::utils};

use hersir::{
    builder::{NetworkBuilder, Node, Topology},
    config::{BlockchainBuilder, BlockchainConfiguration, SpawnParams, WalletTemplateBuilder},
};
use jormungandr_automation::{
    jormungandr::LogLevel,
    testing::{ensure_nodes_are_in_sync, SyncWaitParams},
};

use jormungandr_lib::{
    interfaces::SlotDuration,
    time::{Duration, SystemTime},
};
use std::{
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    path::PathBuf,
};
use thor::{FragmentSender, FragmentVerifier};

const CLIENT: &str = "CLIENT";
const CLIENT_B: &str = "CLIENT_B";
const CLIENT_C: &str = "CLIENT_C";

const SERVER: &str = "SERVER";

const ALICE: &str = "ALICE";
const BOB: &str = "BOB";

const LEADER_CLIENT: &str = "LEADER_CLIENT";
const LEADER: &str = "LEADER";

#[ignore]
#[test]
fn gateway_gossip() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;
    const DEFAULT_GOSSIP_INTERVAL_SECS: u64 = 10;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(SERVER))
                .with_node(Node::new(CLIENT).with_trusted_peer(SERVER))
                .with_node(Node::new(CLIENT_B).with_trusted_peer(SERVER)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(SERVER))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(CLIENT)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(SERVER)
                .build(),
        )
        .build()
        .unwrap();

    let server = network_controller
        .spawn(
            SpawnParams::new(SERVER)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    //
    // spin up gateway node
    //
    let mut gateway_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.push("target/debug/gateway");

    let _client_gateway = network_controller
        .spawn(SpawnParams::new(CLIENT).jormungandr(gateway_node_binary))
        .unwrap();

    utils::wait(DEFAULT_GOSSIP_INTERVAL_SECS);

    //
    // there should be no gossip from gateway node to server
    //
    let last_gossip = server.rest().network_stats().unwrap();

    assert!(last_gossip.is_empty());
    //
    // spin up regular client
    //
    let mut regular_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.push("target/debug/jormungandr");

    let _client_b = network_controller
        .spawn(SpawnParams::new(CLIENT_B).jormungandr(regular_node_binary))
        .unwrap();

    utils::wait(DEFAULT_GOSSIP_INTERVAL_SECS);
    let last_gossip = server.rest().network_stats().unwrap();

    // regular node gossips as usual
    assert!(!last_gossip.is_empty());
}

#[ignore]
#[test]
pub fn test_gateway_node_not_publishing() {
    let mut network_controller = build_network!()
        .topology(
            Topology::default()
                .with_node(Node::new(LEADER))
                .with_node(Node::new(LEADER_CLIENT).with_trusted_peer(LEADER)),
        )
        .wallet_template(
            WalletTemplateBuilder::new("alice")
                .with(1_000_000)
                .delegated_to(LEADER)
                .build(),
        )
        .wallet_template(WalletTemplateBuilder::new("bob").with(1_000_000).build())
        .blockchain_config(BlockchainConfiguration::default().with_leader(LEADER))
        .build()
        .unwrap();

    let leader = network_controller
        .spawn(SpawnParams::new(LEADER).in_memory())
        .unwrap();

    //
    // spin up gateway node
    //
    let mut gateway_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.push("target/debug/gateway");

    let leader_client = network_controller
        .spawn(
            SpawnParams::new(LEADER_CLIENT)
                .jormungandr(gateway_node_binary)
                .listen_address(Some(SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::new(0x26, 0, 0x1c9, 0, 0, 0xafc8, 0x10, 0x1),
                    1234,
                    0,
                    0,
                )))),
        )
        .unwrap();

    let mut alice = network_controller.controlled_wallet("alice").unwrap();
    let mut bob = network_controller.controlled_wallet("bob").unwrap();

    let fragment_sender = FragmentSender::from(&leader.rest().settings().unwrap());

    // gateway node should not be able to send fragments
    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &leader_client,
        100.into(),
    ) {
        Ok(_) => panic!("gateway node should not be able to send fragments"),
        Err(err) => assert_eq!(
            err.to_string(),
            "Too many attempts failed (1) while trying to send fragment to node: ".to_string()
        ),
    };
}

#[ignore]
#[test]
pub fn test_gateway_sync() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;
    const DEFAULT_GOSSIP_INTERVAL_SECS: u64 = 10;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(SERVER))
                .with_node(Node::new(CLIENT).with_trusted_peer(SERVER))
                .with_node(Node::new(CLIENT_B).with_trusted_peer(SERVER))
                .with_node(Node::new(CLIENT_C).with_trusted_peer(SERVER)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(SERVER))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(CLIENT)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(SERVER)
                .build(),
        )
        .build()
        .unwrap();

    let server = network_controller
        .spawn(
            SpawnParams::new(SERVER)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    //
    // spin up gateway node
    //
    let mut gateway_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.pop();
    gateway_node_binary.push("target/debug/gateway");

    let _client_gateway = network_controller
        .spawn(SpawnParams::new(CLIENT).jormungandr(gateway_node_binary))
        .unwrap();

    utils::wait(DEFAULT_GOSSIP_INTERVAL_SECS);

    //
    // spin up regular client
    //
    let mut regular_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.push("target/debug/jormungandr");

    // binary without drop gossip from public addrs modifications
    let regular_node_path = regular_node_binary.clone();

    let _client_b = network_controller
        .spawn(SpawnParams::new(CLIENT_B).jormungandr(regular_node_binary))
        .unwrap();

    let _client_c = network_controller
        .spawn(SpawnParams::new(CLIENT_C).jormungandr(regular_node_path))
        .unwrap();

    utils::wait(DEFAULT_GOSSIP_INTERVAL_SECS);

    let fragment_sender = FragmentSender::from(&server.rest().settings().unwrap());

    let mut alice = network_controller.controlled_wallet(ALICE).unwrap();
    let mut bob = network_controller.controlled_wallet(BOB).unwrap();

    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &_client_b,
        100.into(),
    ) {
        Ok(()) => println!("fragments sent!"),
        Err(err) => panic!("{:?}", err),
    }

    let a = _client_b.grpc().tip().block_content_hash();
    let b = _client_gateway.grpc().tip().block_content_hash();
    let c = server.grpc().tip().block_content_hash();

    assert_eq!(a, b);
    assert_eq!(b, c);

    ensure_nodes_are_in_sync(
        SyncWaitParams::ZeroWait,
        &[&_client_gateway, &_client_b, &_client_c],
    )
    .unwrap();
}
