use jormungandr_automation::jormungandr::{JormungandrRest, NodeAlias, Status};
use jormungandr_automation::testing::NamedProcess;
use valgrind::{ProxyClient, ValgrindClient, ValgrindSettings};

pub type VitStationSettings = vit_servicing_station_lib::server::settings::ServiceSettings;
use std::process::Child;
use std::sync::{Arc, Mutex};

use super::settings::WalletProxySettings;
/// send query to a running node
pub struct WalletProxyController {
    pub(crate) alias: NodeAlias,
    pub(crate) settings: WalletProxySettings,
    pub(crate) status: Arc<Mutex<Status>>,
    pub(crate) process: Child,
    pub(crate) client: ProxyClient,
}

impl WalletProxyController {
    pub fn new(
        alias: NodeAlias,
        settings: WalletProxySettings,
        status: Arc<Mutex<Status>>,
        process: Child,
    ) -> Self {
        let address = settings.address();
        Self {
            alias,
            settings,
            status,
            process,
            client: ProxyClient::new(address),
        }
    }

    pub fn client(&self) -> ValgrindClient {
        let settings = ValgrindSettings {
            use_https: false,
            enable_debug: true,
            certificate: None,
            ..Default::default()
        };

        let base_address = self.settings().base_address();

        ValgrindClient::new_from_addresses(
            base_address.to_string(),
            base_address.to_string(),
            base_address.to_string(),
            settings,
        )
    }

    pub fn alias(&self) -> &NodeAlias {
        &self.alias
    }

    pub fn status(&self) -> Status {
        // FIXME: this is basically a Clone, but it has to be implemented in
        // jormungandr_automatation, this is only just for the sake of making it compile
        match *self.status.lock().unwrap() {
            Status::Running => Status::Running,
            Status::Starting => Status::Starting,
            Status::Exited(e) => Status::Exited(e),
        }
    }

    pub fn check_running(&self) -> bool {
        self.status() == Status::Running
    }

    pub fn block0(&self) -> Vec<u8> {
        self.client.block0().unwrap()
    }

    pub fn address(&self) -> String {
        self.settings.address()
    }

    pub fn rest(&self) -> JormungandrRest {
        JormungandrRest::new(self.address())
    }

    pub fn std_err(&self) -> std::option::Option<std::process::ChildStderr> {
        self.process.stderr
    }

    pub fn as_named_process(&self) -> NamedProcess {
        NamedProcess::new(self.alias().to_string(), self.process.id() as usize)
    }

    pub fn settings(&self) -> &WalletProxySettings {
        &self.settings
    }

    pub fn shutdown(mut self) {
        let _ = self.process.kill();
    }
}
