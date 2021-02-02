use crate::backend::ProxyServerStub;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxProxyCommandError {
    #[error("proxy error")]
    ProxyError(#[from] crate::backend::ProxyServerError),
    #[error("both --cert and --key parametrs need to be defined in order to use https")]
    UnsufficientHttpConfiguration,
    #[error("cert file does not exists")]
    CertFileDoesNotExist,
    #[error("key file does not exists")]
    KeyFileDoesNotExist,
}

#[derive(StructOpt, Debug)]
pub struct IapyxProxyCommand {
    #[structopt(short = "a", long = "address", default_value = "127.0.0.1:8000")]
    pub address: String,

    #[structopt(short = "v", long = "vit-address", default_value = "127.0.0.1:3030")]
    pub vit_address: String,

    #[structopt(short = "n", long = "node-address", default_value = "127.0.0.1:8080")]
    pub node_address: String,

    #[structopt(short = "b", long = "block0")]
    pub block0_path: PathBuf,

    #[structopt(long = "cert")]
    pub cert_path: Option<PathBuf>,
    
    #[structopt(long = "key")]
    pub key_path: Option<PathBuf>,
 
}

impl IapyxProxyCommand {
    pub fn build(&self) -> Result<ProxyServerStub, IapyxProxyCommandError> {
        let proxy_address = self.address.clone();
        let vit_address = self.vit_address.clone();
        let node_address = self.node_address.clone();
        let block0_path = self.block0_path.clone();


        if let Some(cert_path) = &self.cert_path {
    
            let key_path = self.key_path.clone().ok_or(IapyxProxyCommandError::UnsufficientHttpConfiguration)?;

            if !key_path.exists() {
                return Err(IapyxProxyCommandError::KeyFileDoesNotExist);
            }

            if !cert_path.exists() {
                return Err(IapyxProxyCommandError::CertFileDoesNotExist);
            }

            return Ok(ProxyServerStub::new_https(
                key_path,
                cert_path.to_path_buf(),
                proxy_address,
                vit_address,
                node_address,
                jortestkit::file::get_file_as_byte_vec(&block0_path),
            ));

        } 

        Ok(ProxyServerStub::new_http(
            proxy_address,
            vit_address,
            node_address,
            jortestkit::file::get_file_as_byte_vec(&block0_path),
        ))
    }
}
