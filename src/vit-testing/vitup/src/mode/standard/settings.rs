use super::{VitStationSettings, WalletProxySettings};
use crate::config::Blockchain;
use hersir::{
    builder::{NodeAlias, Topology},
    config::SessionSettings,
};
use jormungandr_automation::jormungandr::get_available_port;
use std::collections::HashMap;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_tests::common::startup::server::ServerSettingsBuilder;

pub trait PrepareVitServerSettings: Clone + Send {
    fn prepare(session_settings: &mut SessionSettings) -> Self;
}

pub trait PrepareWalletProxySettings: Clone + Send {
    fn prepare(
        session_settings: &mut SessionSettings,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self;
}

pub trait PrepareSettings {
    fn prepare(
        topology: Topology,
        blockchain: Blockchain,
        session_settings: &mut SessionSettings,
    ) -> Self;
}

#[derive(Debug, Clone)]
pub struct VitSettings {
    pub vit_stations: HashMap<NodeAlias, ServiceSettings>,
    pub wallet_proxies: HashMap<NodeAlias, WalletProxySettings>,
}

impl VitSettings {
    pub fn new(session_settings: &mut SessionSettings) -> Self {
        let mut vit_stations = HashMap::new();
        let vit_station = VitStationSettings::prepare(session_settings);
        vit_stations.insert("vit_station".to_string(), vit_station);

        let mut wallet_proxies = HashMap::new();
        let wallet_proxy_setting = WalletProxySettings::prepare(session_settings, &vit_stations);
        wallet_proxies.insert("wallet_proxy".to_string(), wallet_proxy_setting);

        VitSettings {
            vit_stations,
            wallet_proxies,
        }
    }
}

impl PrepareVitServerSettings for VitStationSettings {
    fn prepare(_session_settings: &mut SessionSettings) -> Self {
        let mut settings_builder: ServerSettingsBuilder = Default::default();
        settings_builder
            .with_localhost_address(get_available_port() as u32)
            .build()
    }
}
