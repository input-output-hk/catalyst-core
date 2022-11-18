mod node;

use crate::jormungandr::starter::{CommunicationParams, ConfigurableNodeConfig};
use jormungandr_lib::multiaddr::to_tcp_socket_addr;
use multiaddr::Multiaddr;
pub use node::{
    Error as LegacyConfigError, LegacyNodeConfig, LegacyNodeConfigBuilder,
    LegacyNodeConfigConverter,
};
use serde::Serialize;
use std::{
    fmt::{Debug, Formatter},
    net::SocketAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize)]
pub struct LegacyNodeConfigManager {
    pub node_config: LegacyNodeConfig,
    pub file: Option<PathBuf>,
}

impl Debug for LegacyNodeConfigManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.node_config.fmt(f)
    }
}

impl ConfigurableNodeConfig for LegacyNodeConfigManager {
    fn log_file_path(&self) -> Option<&Path> {
        todo!()
    }

    fn write_node_config(&self) {
        todo!()
    }

    fn node_config_path(&self) -> PathBuf {
        self.file
            .as_ref()
            .expect("no legacy config path defined")
            .clone()
    }

    fn set_node_config_path(&mut self, path: PathBuf) {
        self.file = Some(path);
    }

    fn p2p_listen_address(&self) -> SocketAddr {
        self.node_config
            .p2p
            .listen
            .unwrap_or_else(|| to_tcp_socket_addr(&self.node_config.p2p.public_address).unwrap())
    }

    fn p2p_public_address(&self) -> Multiaddr {
        self.node_config.p2p.public_address.clone()
    }

    fn set_p2p_public_address(&mut self, address: Multiaddr) {
        self.node_config.p2p.public_address = address;
    }

    fn rest_socket_addr(&self) -> SocketAddr {
        self.node_config.rest.listen
    }

    fn set_rest_socket_addr(&mut self, addr: SocketAddr) {
        self.node_config.rest.listen = addr;
    }

    fn as_communication_params(&self) -> CommunicationParams {
        CommunicationParams {
            p2p_public_address: self.p2p_public_address(),
            p2p_listen_address: self.p2p_listen_address(),
            rest_socket_addr: self.rest_socket_addr(),
        }
    }
}
