use jormungandr_automation::jormungandr::{NodeAlias, Status};
use jormungandr_automation::testing::NamedProcess;
use std::process::Child;
use std::sync::{Arc, Mutex};
use valgrind::{ProxyClient, ValgrindClient, ValgrindSettings};

use super::settings::WalletProxySettings;
/// send query to a running node
pub struct WalletProxyController {
    pub(crate) alias: NodeAlias,
    pub(crate) settings: WalletProxySettings,
    pub(crate) status: Arc<Mutex<Status>>,
    pub(crate) process: Child,
    pub(crate) client: ProxyClient,
    pub(crate) valgrind: ValgrindClient,
}

impl WalletProxyController {
    pub fn new(
        alias: NodeAlias,
        settings: WalletProxySettings,
        status: Arc<Mutex<Status>>,
        process: Child,
    ) -> Result<Self, Error> {
        let address = settings.address();

        let valgrind_settings = ValgrindSettings {
            use_https: false,
            enable_debug: false,
            certificate: None,
            ..Default::default()
        };

        let valgrind = ValgrindClient::new(settings.address(), valgrind_settings)?;
        Ok(Self {
            alias,
            settings,
            status,
            process,
            client: ProxyClient::new(address),
            valgrind,
        })
    }

    pub fn client(&self) -> ValgrindClient {
        self.valgrind.clone()
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

    pub fn as_named_process(&self) -> NamedProcess {
        NamedProcess::new(self.alias().to_string(), self.process.id() as usize)
    }

    pub fn settings(&self) -> &WalletProxySettings {
        &self.settings
    }

    pub fn shutdown(&mut self) {
        let _ = self.process.kill();
    }
}

impl Drop for WalletProxyController {
    fn drop(&mut self) {
        self.shutdown();
        self.process.wait().unwrap();
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Valgrind(#[from] valgrind::Error),
}
