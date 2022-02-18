mod server;

use serde::{Deserialize, Serialize};
use server::{Error as ProxyServerError, ProxyServerStub};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("proxy error")]
    ProxyError(#[from] ProxyServerError),
    #[error("both --cert and --key parametrs need to be defined in order to use https")]
    UnsufficientHttpConfiguration,
    #[error("cert file does not exists")]
    CertFileDoesNotExist,
    #[error("key file does not exists")]
    KeyFileDoesNotExist,
}

#[derive(StructOpt, Debug)]
pub struct ValigrindStartupCommand {
    #[structopt(short = "a", long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    #[structopt(short = "v", long = "vit-address", default_value = "127.0.0.1:3030")]
    pub vit_address: String,

    #[structopt(short = "n", long = "node-address", default_value = "127.0.0.1:8080")]
    pub node_address: String,

    #[structopt(short = "b", long = "block0")]
    pub block0_path: PathBuf,

    #[structopt(long = "cert")]
    pub cert_path: PathBuf,

    #[structopt(long = "key")]
    pub key_path: PathBuf,
}

impl ValigrindStartupCommand {
    pub fn build(self) -> Result<ProxyServerStub, Error> {
        let Self {
            address,
            vit_address,
            node_address,
            block0_path,
            key_path,
            cert_path,
        } = self;

        if !key_path.exists() {
            return Err(Error::KeyFileDoesNotExist);
        }

        if !cert_path.exists() {
            return Err(Error::CertFileDoesNotExist);
        }

        Ok(ProxyServerStub::new(
            key_path,
            cert_path,
            address,
            vit_address,
            node_address,
            jortestkit::file::get_file_as_byte_vec(&block0_path),
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certs {
    pub key_path: PathBuf,
    pub cert_path: PathBuf,
}
