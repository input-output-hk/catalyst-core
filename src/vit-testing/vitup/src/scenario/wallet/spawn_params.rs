use crate::wallet::WalletProxySettings;
use iapyx::Protocol;
use std::net::SocketAddr;

pub struct WalletProxySpawnParams {
    pub alias: String,
    pub base_address: Option<SocketAddr>,
    pub protocol: Protocol,
}

impl WalletProxySpawnParams {
    pub fn new<S: Into<String>>(alias: S) -> Self {
        Self {
            alias: alias.into(),
            base_address: None,
            protocol: Protocol::Http,
        }
    }

    pub fn with_base_address<S: Into<String>>(&mut self, base_address: S) -> &mut Self {
        self.base_address = Some(base_address.into().parse().unwrap());
        self
    }

    pub fn with_protocol(&mut self, protocol: Protocol) -> &mut Self {
        self.protocol = protocol;
        self
    }

    pub fn override_settings(&self, settings: &mut WalletProxySettings) {
        if let Some(address) = self.base_address {
            settings.proxy_address = address;
        }
    }
}
