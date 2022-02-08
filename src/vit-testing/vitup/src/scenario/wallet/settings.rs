use crate::scenario::{
    settings::PrepareWalletProxySettings, vit_station::VitStationSettings, wallet::NodeAlias,
};
use hersir::config::SessionSettings;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct WalletProxySettings {
    pub proxy_address: SocketAddr,
    pub vit_station_address: SocketAddr,
    pub node_backend_address: Option<SocketAddr>,
}

impl WalletProxySettings {
    pub fn base_address(&self) -> SocketAddr {
        self.proxy_address
    }

    pub fn base_vit_address(&self) -> SocketAddr {
        self.vit_station_address
    }

    pub fn base_node_backend_address(&self) -> Option<SocketAddr> {
        self.node_backend_address
    }

    pub fn address(&self) -> String {
        format!("http://{}", self.base_address())
    }

    pub fn vit_address(&self) -> String {
        format!("http://{}", self.base_vit_address())
    }

    pub fn node_backend_address(&self) -> String {
        format!(
            "http://{}/api/v0",
            self.base_node_backend_address().unwrap()
        )
    }
}

impl PrepareWalletProxySettings for WalletProxySettings {
    fn prepare(
        _session_settings: &mut SessionSettings,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self {
        let vit_station_settings = vit_stations
            .values()
            .next()
            .expect("no vit stations defined");

        WalletProxySettings {
            proxy_address: "127.0.0.1:8080".parse().unwrap(),
            vit_station_address: vit_station_settings.address,
            node_backend_address: None,
        }
    }
}
