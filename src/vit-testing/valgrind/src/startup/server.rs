use crate::startup::Certs;
use std::net::SocketAddr;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Malformed proxy address: {0}")]
    Proxy(String),
    #[error("Malformed vit address: {0}")]
    VitStation(String),
    #[error("Malformed node rest address: {0}")]
    NodeRest(String),
}

pub struct ProxyServerStub {
    certs: Certs,
    address: String,
    vit_address: String,
    node_rest_address: String,
    block0: Vec<u8>,
}

impl ProxyServerStub {
    pub fn new(
        key_path: PathBuf,
        cert_path: PathBuf,
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
    ) -> Self {
        Self {
            certs: Certs {
                key_path,
                cert_path,
            },
            address,
            vit_address,
            node_rest_address,
            block0,
        }
    }

    pub fn certs(&self) -> &Certs {
        &self.certs
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
