use crate::{
    build_network,
    jormungandr::grpc::setup::client::{self, default},
    networking::utils,
};

use hersir::{
    builder::{NetworkBuilder, Node, Topology},
    config::{
        BlockchainBuilder, BlockchainConfiguration, NodeConfig, SpawnParams, WalletTemplateBuilder,
    },
};
use indicatif::MultiProgress;
use jormungandr_automation::{
    jormungandr::{explorer::configuration::ExplorerParams, LogLevel},
    testing::{ensure_nodes_are_in_sync, SyncWaitParams},
};

use jormungandr_lib::{
    interfaces::{Policy, PreferredListConfig, SlotDuration, TrustedPeer},
    time::{Duration, SystemTime},
};
use multiaddr::Multiaddr;
use std::{
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    path::PathBuf,
};
use thor::{FragmentSender, FragmentVerifier};

const SERVER: &str = "SERVER";
const GATEWAY: &str = "GATEWAY_NODE";
const PUBLIC_NODE: &str = "PUBLIC";
const INTERNAL_NODE: &str = "INTERNAL";

const ALICE: &str = "ALICE";
const BOB: &str = "BOB";

#[ignore]
#[test]
fn gateway_gossip() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(GATEWAY))
                .with_node(Node::new(INTERNAL_NODE).with_trusted_peer(GATEWAY))
                .with_node(Node::new(PUBLIC_NODE).with_trusted_peer(GATEWAY)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(GATEWAY))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(GATEWAY)
                .build(),
        )
        .build()
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

    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .jormungandr(gateway_node_binary)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    utils::wait(10);

    //
    // spin up regular client outside the intranet
    //
    let mut regular_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.push("target/debug/jormungandr");

    let regular_node = regular_node_binary.clone();

    let gateway_addr = network_controller
        .node_config(GATEWAY)
        .unwrap()
        .p2p
        .public_address;

    let policy = Policy {
        quarantine_duration: Some(Duration::new(600, 0)),
        quarantine_whitelist: Some(vec![gateway_addr.clone()]),
    };

    let t = TrustedPeer {
        id: None,
        address: gateway_addr,
    };

    let mut preferred = PreferredListConfig::default();
    preferred.peers = vec![t];

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .preferred_layer(preferred.clone())
                .policy(policy.clone())
                .jormungandr(regular_node)
                .gossip_interval(Duration::new(1, 0)),
        )
        .unwrap();

    utils::wait(20);

    // spin up node within the intranet
    // should not receive gossip from public node

    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .policy(policy.clone())
                .preferred_layer(preferred)
                .jormungandr(regular_node_binary)
                .gossip_interval(Duration::new(1, 0))
                .public_address(address)
                .max_connections(1),
        )
        .unwrap();

    utils::wait(10);

    // internal node should not receive gossip from public node

    let is_gossiping_with_public_node = _client_internal
        .logger
        .get_lines_as_string()
        .iter()
        .any(|s| s.contains("80.9.12.3"));

    utils::wait(10);

    println!("{:?}", _client_internal.logger.get_lines_as_string());

    assert!(!is_gossiping_with_public_node);
}

#[ignore]
#[test]
pub fn test_public_node_not_publishing() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(GATEWAY))
                .with_node(Node::new(INTERNAL_NODE).with_trusted_peer(GATEWAY))
                .with_node(Node::new(PUBLIC_NODE)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(SERVER))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(GATEWAY)
                .build(),
        )
        .build()
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

    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .jormungandr(gateway_node_binary)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    utils::wait(10);

    //
    // spin up regular client outside the intranet
    //
    let mut regular_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.push("target/debug/jormungandr");

    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let regular_node = regular_node_binary.clone();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .jormungandr(regular_node_binary)
                .gossip_interval(Duration::new(2, 0))
                .public_address(address),
        )
        .unwrap();

    // spin up node within the intranet

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .jormungandr(regular_node)
                .gossip_interval(Duration::new(3, 0)),
        )
        .unwrap();

    let mut alice = network_controller.controlled_wallet(ALICE).unwrap();
    let mut bob = network_controller.controlled_wallet(BOB).unwrap();

    let fragment_sender = FragmentSender::from(&_gateway.rest().settings().unwrap());

    // public node should not be able to send fragments
    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &_client_public,
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

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(GATEWAY))
                .with_node(Node::new(INTERNAL_NODE).with_trusted_peer(GATEWAY))
                .with_node(Node::new(PUBLIC_NODE)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(SERVER))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(GATEWAY)
                .build(),
        )
        .build()
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

    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .jormungandr(gateway_node_binary)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    utils::wait(10);

    //
    // spin up regular client outside the intranet
    //
    let mut regular_node_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.pop();
    regular_node_binary.push("target/debug/jormungandr");

    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let regular_node = regular_node_binary.clone();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .jormungandr(regular_node_binary)
                .gossip_interval(Duration::new(2, 0))
                .public_address(address),
        )
        .unwrap();

    // spin up node within the intranet

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .jormungandr(regular_node)
                .gossip_interval(Duration::new(3, 0)),
        )
        .unwrap();

    let mut alice = network_controller.controlled_wallet(ALICE).unwrap();
    let mut bob = network_controller.controlled_wallet(BOB).unwrap();

    let fragment_sender = FragmentSender::from(&_gateway.rest().settings().unwrap());

    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &_client_internal,
        100.into(),
    ) {
        Ok(_) => println!("fragments sent!"),
        Err(err) => assert_eq!(
            err.to_string(),
            "Too many attempts failed (1) while trying to send fragment to node: ".to_string()
        ),
    };

    utils::wait(5);

    let internal_node_tip = _client_internal.grpc().tip();

    // public node is up to date
    let block = _client_public
        .rest()
        .block(&internal_node_tip.hash())
        .unwrap();

    assert_eq!(
        block.header().block_content_hash(),
        internal_node_tip.block_content_hash()
    );

    let public_node_syncs_with_internal_network = _client_internal
        .logger
        .get_lines_as_string()
        .iter()
        .any(|s| s.contains("receiving block stream from network"));

    assert!(public_node_syncs_with_internal_network);
}
