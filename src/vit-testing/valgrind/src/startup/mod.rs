mod server;

use clap::Parser;
use serde::{Deserialize, Serialize};
use server::{Error as ProxyServerError, ProxyServerStub};
use std::path::PathBuf;
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
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Parser, Debug)]
pub struct ValigrindStartupCommand {
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    #[clap(short = 'v', long = "vit-address", default_value = "127.0.0.1:3030")]
    pub vit_address: String,

    #[clap(short = 'n', long = "node-address", default_value = "127.0.0.1:8080")]
    pub node_address: String,

    #[clap(short = 'b', long = "block0")]
    pub block0_path: PathBuf,

    #[clap(long = "cert")]
    pub cert_path: Option<PathBuf>,

    #[clap(long = "key")]
    pub key_path: Option<PathBuf>,
}

impl ValigrindStartupCommand {
    pub fn build(self) -> Result<ProxyServerStub, Error> {
        let proxy_address = self.address.clone();
        let vit_address = self.vit_address.clone();
        let node_address = self.node_address.clone();
        let block0_path = self.block0_path.clone();

        if let Some(cert_path) = self.cert_path {
            let key_path = self
                .key_path
                .clone()
                .ok_or(Error::UnsufficientHttpConfiguration)?;

            if !key_path.exists() {
                return Err(Error::KeyFileDoesNotExist);
            }

            if !cert_path.exists() {
                return Err(Error::CertFileDoesNotExist);
            }

            let certs = Certs {
                key_path,
                cert_path,
            };

            return Ok(ProxyServerStub::new_https(
                certs,
                proxy_address,
                vit_address,
                node_address,
                jortestkit::file::get_file_as_byte_vec(&block0_path)?,
            ));
        }

        Ok(ProxyServerStub::new_http(
            proxy_address,
            vit_address,
            node_address,
            jortestkit::file::get_file_as_byte_vec(&block0_path)?,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Protocol {
    #[default]
    Http,
    Https(Certs),
}

impl Protocol {
    pub fn schema(&self) -> String {
        match self {
            Self::Http => "http".to_string(),
            Self::Https { .. } => "https".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certs {
    pub key_path: PathBuf,
    pub cert_path: PathBuf,
}

impl From<Certs> for Protocol {
    fn from(certs: Certs) -> Self {
        Self::Https(certs)
    }
}
