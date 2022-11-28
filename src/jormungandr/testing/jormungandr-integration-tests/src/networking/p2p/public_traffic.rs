use crate::networking::utils;
use hersir::{
    builder::{NetworkBuilder, Node, Topology},
    config::{
        BlockchainBuilder, BlockchainConfiguration, NodeConfig, SpawnParams, WalletTemplateBuilder,
    },
};
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

const GATEWAY: &str = "GATEWAY";

const PUBLIC_NODE: &str = "PUBLIC";
const INTERNAL_NODE: &str = "INTERNAL";
const INTERNAL_NODE_2: &str = "INTERNAL_2";
const INTERNAL_NODE_3: &str = "INTERNAL_3";
const INTERNAL_NODE_4: &str = "INTERNAL_4";
const LEADER: &str = "LEADER";

const ALICE: &str = "ALICE";
const BOB: &str = "BOB";

#[ignore]
#[test]
fn public_gossip_rejection() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 10;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(INTERNAL_NODE))
                .with_node(Node::new(GATEWAY).with_trusted_peer(INTERNAL_NODE))
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

    // spin up node within the intranet
    // gossip from public node should be dropped

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .gossip_interval(Duration::new(5, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    // node from internal network exposed to public
    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .allow_private_addresses(true)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    // simulate node in the wild
    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .gossip_interval(Duration::new(5, 0))
                .public_address(address)
                .allow_private_addresses(true),
        )
        .unwrap();

    utils::wait(20);

    let mut gossip_dropped = false;
    // internal node should drop gossip from public node
    for i in _client_internal.logger.get_lines_as_string().into_iter() {
        if i.contains("nodes dropped from gossip") && i.contains("80.9.12.3") {
            gossip_dropped = true
        }
    }

    assert!(gossip_dropped);
}

#[test]
pub fn test_node_sync() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(INTERNAL_NODE))
                .with_node(Node::new(INTERNAL_NODE_2).with_trusted_peer(INTERNAL_NODE))
                .with_node(Node::new(GATEWAY).with_trusted_peer(INTERNAL_NODE_2))
                .with_node(Node::new(PUBLIC_NODE).with_trusted_peer(GATEWAY)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(INTERNAL_NODE))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE_2)
                .build(),
        )
        .build()
        .unwrap();

    // spin up node within the intranet
    // gossip from public node should be dropped

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .gossip_interval(Duration::new(1, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    let _client_internal_2 = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE_2)
                .gossip_interval(Duration::new(1, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    // node from internal network exposed to public
    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .allow_private_addresses(true)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    // simulate node in the wild
    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .gossip_interval(Duration::new(1, 0))
                .public_address(address)
                .allow_private_addresses(true),
        )
        .unwrap();

    let mut alice = network_controller.controlled_wallet(ALICE).unwrap();
    let mut bob = network_controller.controlled_wallet(BOB).unwrap();

    let fragment_sender = FragmentSender::from(&_client_internal.rest().settings().unwrap());

    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &_client_internal,
        100.into(),
    ) {
        Ok(_) => println!("fragments sent!"),
        Err(err) => panic!("{:?}", err),
    };

    utils::wait(30);

    ensure_nodes_are_in_sync(
        SyncWaitParams::ZeroWait,
        &[&_gateway, &_client_internal, &_client_internal_2],
    )
    .unwrap();
}

#[test]
pub fn test_public_node_cannot_publish() {
    const SERVER_GOSSIP_INTERVAL_SECS: u64 = 1;

    let mut network_controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(INTERNAL_NODE))
                .with_node(Node::new(INTERNAL_NODE_2).with_trusted_peer(INTERNAL_NODE))
                .with_node(Node::new(INTERNAL_NODE_3).with_trusted_peer(INTERNAL_NODE_2))
                .with_node(Node::new(GATEWAY).with_trusted_peer(INTERNAL_NODE))
                .with_node(Node::new(PUBLIC_NODE).with_trusted_peer(GATEWAY)),
        )
        .blockchain_config(BlockchainConfiguration::default().with_leader(INTERNAL_NODE))
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(1_000_000)
                .delegated_to(INTERNAL_NODE_2)
                .build(),
        )
        .build()
        .unwrap();

    // spin up node within the intranet
    // gossip from public node should be dropped

    let _client_internal = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE)
                .gossip_interval(Duration::new(1, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    let _client_internal_2 = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE_2)
                .gossip_interval(Duration::new(1, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    let _client_internal_3 = network_controller
        .spawn(
            SpawnParams::new(INTERNAL_NODE_3)
                .gossip_interval(Duration::new(1, 0))
                .allow_private_addresses(false),
        )
        .unwrap();

    // node from internal network exposed to public
    let _gateway = network_controller
        .spawn(
            SpawnParams::new(GATEWAY)
                .gossip_interval(Duration::new(SERVER_GOSSIP_INTERVAL_SECS, 0))
                .allow_private_addresses(true)
                .log_level(LogLevel::TRACE),
        )
        .unwrap();

    // simulate node in the wild
    let address: Multiaddr = "/ip4/80.9.12.3/tcp/0".parse().unwrap();

    let _client_public = network_controller
        .spawn(
            SpawnParams::new(PUBLIC_NODE)
                .gossip_interval(Duration::new(1, 0))
                .public_address(address)
                .allow_private_addresses(false),
        )
        .unwrap();

    let mut alice = network_controller.controlled_wallet(ALICE).unwrap();
    let mut bob = network_controller.controlled_wallet(BOB).unwrap();

    // public node sends fragments to network
    let fragment_sender = FragmentSender::from(&network_controller.settings().block0);

    match fragment_sender.send_transactions_round_trip(
        5,
        &mut alice,
        &mut bob,
        &_client_public,
        100.into(),
    ) {
        Ok(_) => println!("fragments sent!"),
        Err(err) => panic!("{:?}", err),
    };

    // public node should not propagate state

    println!(
        "a {:?}  {:?}",
        _client_public.rest().account_state(&alice.account_id()),
        _client_public.rest().account_state(&bob.account_id()),
    );

    println!(
        "a {:?}  {:?}",
        _client_internal.rest().account_state(&alice.account_id()),
        _client_internal.rest().account_state(&bob.account_id()),
    );
}
