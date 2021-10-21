use crate::vit_station::VitStationSettings;
use jormungandr_scenario_tests::scenario::Context;

use jormungandr_testing_utils::testing::network::WalletProxySettings;
use jormungandr_testing_utils::testing::network::{
    Blockchain as BlockchainTemplate, NodeAlias, Topology as TopologyTemplate,
};
use rand_core::{CryptoRng, RngCore};
use std::collections::HashMap;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_tests::common::startup::server::ServerSettingsBuilder;

pub trait PrepareVitServerSettings: Clone + Send {
    fn prepare<RNG>(context: &mut Context<RNG>) -> Self
    where
        RNG: RngCore + CryptoRng;
}

pub trait PrepareWalletProxySettings: Clone + Send {
    fn prepare<RNG>(
        context: &mut Context<RNG>,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self
    where
        RNG: RngCore + CryptoRng;
}

pub trait PrepareSettings {
    fn prepare<RNG>(
        topology: TopologyTemplate,
        blockchain: BlockchainTemplate,
        context: &mut Context<RNG>,
    ) -> Self
    where
        RNG: RngCore + CryptoRng;
}

#[derive(Debug)]
pub struct VitSettings {
    pub vit_stations: HashMap<NodeAlias, ServiceSettings>,
    pub wallet_proxies: HashMap<NodeAlias, WalletProxySettings>,
}

impl VitSettings {
    pub fn new<RNG>(context: &mut Context<RNG>) -> Self
    where
        RNG: RngCore + CryptoRng,
    {
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
    fn prepare<RNG>(context: &mut Context<RNG>) -> Self
    where
        RNG: RngCore + CryptoRng,
    {
        let mut settings_builder: ServerSettingsBuilder = Default::default();
        settings_builder
            .with_localhost_address(context.generate_new_unique_port() as u32)
            .build()
    }
}
