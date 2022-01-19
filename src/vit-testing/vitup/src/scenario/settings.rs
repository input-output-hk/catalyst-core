use super::wallet::WalletProxySettings;
use crate::vit_station::VitStationSettings;
use hersir::{
    builder::{Blockchain, NodeAlias, Topology},
    controller::Context,
};
use jormungandr_automation::jormungandr::get_available_port;
use std::collections::HashMap;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_tests::common::startup::server::ServerSettingsBuilder;

pub trait PrepareVitServerSettings: Clone + Send {
    fn prepare(context: &mut Context) -> Self;
}

pub trait PrepareWalletProxySettings: Clone + Send {
    fn prepare(
        context: &mut Context,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self;
}

pub trait PrepareSettings {
    fn prepare(topology: Topology, blockchain: Blockchain, context: &mut Context) -> Self;
}

#[derive(Debug)]
pub struct VitSettings {
    pub vit_stations: HashMap<NodeAlias, ServiceSettings>,
    pub wallet_proxies: HashMap<NodeAlias, WalletProxySettings>,
}

impl VitSettings {
    pub fn new(context: &mut Context) -> Self {
        let mut vit_stations = HashMap::new();
        let vit_station = VitStationSettings::prepare(context);
        vit_stations.insert("vit_station".to_string(), vit_station);

        let mut wallet_proxies = HashMap::new();
        let wallet_proxy_setting = WalletProxySettings::prepare(context, &vit_stations);
        wallet_proxies.insert("wallet_proxy".to_string(), wallet_proxy_setting);

        VitSettings {
            vit_stations,
            wallet_proxies,
        }
    }
}

impl PrepareVitServerSettings for VitStationSettings {
    fn prepare(_context: &mut Context) -> Self {
        let mut settings_builder: ServerSettingsBuilder = Default::default();
        settings_builder
            .with_localhost_address(get_available_port() as u32)
            .build()
    }
}
