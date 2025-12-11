use super::{VitStationSettings, WalletProxySettings};
use crate::builders::ArchiveConfiguration;
use crate::config::Blockchain;
use hersir::{
    builder::{NodeAlias, Topology},
    config::SessionSettings,
};
use jormungandr_automation::jormungandr::get_available_port;
use std::collections::HashMap;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_tests::common::startup::server::ServerSettingsBuilder;

pub const VIT_STATION: &str = "vit_station";
pub const VIT_STATION_ARCHIVE: &str = "vit_station_archive";

#[allow(dead_code)]
pub trait PrepareVitServerSettings: Clone + Send {
    fn prepare(session_settings: &mut SessionSettings) -> Self;
}

pub trait PrepareWalletProxySettings: Clone + Send {
    fn prepare(
        session_settings: &mut SessionSettings,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self;
}

#[allow(dead_code)]
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
    pub fn new(
        session_settings: &mut SessionSettings,
        archive_conf: Option<ArchiveConfiguration>,
    ) -> Self {
        let mut vit_stations = HashMap::new();

        let mut settings_builder = ServerSettingsBuilder::default();

        vit_stations.insert(
            VIT_STATION.to_string(),
            settings_builder
                .with_localhost_address(get_available_port() as u32)
                .build(),
        );

        if let Some(archive_conf) = archive_conf {
            let mut settings_builder = ServerSettingsBuilder::default();
            settings_builder
                .with_localhost_address(get_available_port() as u32)
                .with_block0_folder_path(archive_conf.block_folder)
                .with_db_path(archive_conf.archive_db.to_str().unwrap());
            vit_stations.insert(VIT_STATION_ARCHIVE.to_string(), settings_builder.build());
        }

        let mut wallet_proxies = HashMap::new();
        let wallet_proxy_setting = WalletProxySettings::prepare(session_settings, &vit_stations);
        wallet_proxies.insert("wallet_proxy".to_string(), wallet_proxy_setting);

        VitSettings {
            vit_stations,
            wallet_proxies,
        }
    }
}
