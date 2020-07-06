use crate::common::startup::get_exe;
use std::net::SocketAddr;
use std::process::Command;
use std::process::Stdio;
use thiserror::Error;
use vit_servicing_station_lib::server::settings::ServiceSettings;

use crate::common::{paths::BLOCK0_BIN, server::Server, startup::get_available_port};

pub struct ServerBootstrapper {
    settings: ServiceSettings,
}

impl ServerBootstrapper {
    pub fn new() -> Self {
        let mut settings: ServiceSettings = Default::default();
        settings.address = Self::format_localhost_address(get_available_port());
        settings.block0_path = BLOCK0_BIN.to_string();

        Self { settings }
    }

    pub fn with_localhost_address(&mut self, port: u32) -> &mut Self {
        self.settings.address = Self::format_localhost_address(port);
        self
    }

    fn format_localhost_address(port: u32) -> SocketAddr {
        format!("127.0.0.1:{}", port).parse().unwrap()
    }

    pub fn with_db_path<S: Into<String>>(&mut self, db_url: S) -> &mut Self {
        self.settings.db_url = db_url.into();
        self
    }

    pub fn with_block0_path<S: Into<String>>(&mut self, block0_path: S) -> &mut Self {
        self.settings.block0_path = block0_path.into();
        self
    }

    pub fn start(&self) -> Result<Server, ServerBootstrapperError> {
        let child = Command::new(get_exe())
            .arg("--address")
            .arg(self.settings.address.to_string())
            .arg("--db-url")
            .arg(self.settings.db_url.to_string())
            .arg("--block0-path")
            .arg(self.settings.block0_path.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(Server::new(child, self.settings.clone()))
    }
}

impl Default for ServerBootstrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum ServerBootstrapperError {
    #[error("cannot spawn process")]
    ProcessSpawnError(#[from] std::io::Error),
    #[error("cannot find binary (0)")]
    CargoError(#[from] assert_cmd::cargo::CargoError),
}
