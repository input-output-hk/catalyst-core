use crate::mode::standard::{ExplorerController, VitStationController, WalletProxyController};

pub struct VitUserInteractionController {
    vit_stations: Vec<VitStationController>,
    explorers: Vec<ExplorerController>,
    proxies: Vec<WalletProxyController>,
}

impl Default for VitUserInteractionController {
    fn default() -> Self {
        Self::new()
    }
}

impl VitUserInteractionController {
    pub fn new() -> Self {
        Self {
            vit_stations: Vec::new(),
            proxies: Vec::new(),
            explorers: Vec::new(),
        }
    }

    pub fn vit_stations_mut(&mut self) -> &mut Vec<VitStationController> {
        &mut self.vit_stations
    }

    pub fn proxies(&self) -> &[WalletProxyController] {
        &self.proxies
    }

    pub fn vit_stations(&self) -> &[VitStationController] {
        &self.vit_stations
    }

    pub fn explorers(&self) -> &[ExplorerController] {
        &self.explorers
    }

    pub fn proxies_mut(&mut self) -> &mut Vec<WalletProxyController> {
        &mut self.proxies
    }

    pub fn finalize(self) {
        for mut proxy in self.proxies {
            proxy.shutdown();
        }

        for mut vit_station in self.vit_stations {
            vit_station.shutdown();
        }
    }
}
