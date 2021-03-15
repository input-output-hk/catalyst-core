use crate::data::VitVersion;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Malformed proxy address: {0}")]
    MalformedProxyAddress(String),
    #[error("Malformed vit address: {0}")]
    MalformedVitStationAddress(String),
    #[error("Malformed node rest address: {0}")]
    MalformedNodeRestAddress(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Protocol {
    Http,
    Https {
        key_path: PathBuf,
        cert_path: PathBuf,
    },
}

impl Protocol {
    pub fn http() -> Self {
        Self::Http
    }
}

pub struct ProxyServerStub {
    protocol: Protocol,
    address: String,
    vit_address: String,
    node_rest_address: String,
    block0: Vec<u8>,
    vit_version: String,
}

impl ProxyServerStub {
    pub fn new_https(
        key_path: PathBuf,
        cert_path: PathBuf,
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
        vit_version: String,
    ) -> Self {
        Self::new(
            Protocol::Https {
                key_path,
                cert_path,
            },
            address,
            vit_address,
            node_rest_address,
            block0,
            vit_version,
        )
    }

    pub fn new_http(
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
        vit_version: String,
    ) -> Self {
        Self::new(
            Protocol::Http,
            address,
            vit_address,
            node_rest_address,
            block0,
            vit_version,
        )
    }

    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    pub fn new(
        protocol: Protocol,
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
        vit_version: String,
    ) -> Self {
        Self {
            protocol,
            address,
            vit_address,
            node_rest_address,
            block0,
            vit_version,
        }
    }

    pub fn block0(&self) -> Vec<u8> {
        self.block0.clone()
    }

    pub fn address(&self) -> String {
        self.address.parse().unwrap()
    }

    pub fn vit_address(&self) -> String {
        self.vit_address.parse().unwrap()
    }

    pub fn node_rest_address(&self) -> String {
        self.node_rest_address.parse().unwrap()
    }

    pub fn vit_version(&self) -> VitVersion {
        VitVersion::new(self.vit_version.clone())
    }

    pub fn base_address(&self) -> SocketAddr {
        self.address.parse().unwrap()
    }

    pub fn http_vit_address(&self) -> String {
        format!("http://{}/", self.vit_address)
    }

    pub fn http_node_address(&self) -> String {
        format!("http://{}/", self.node_rest_address)
    }
}
