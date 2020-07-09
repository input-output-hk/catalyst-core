use std::process::Stdio;
use thiserror::Error;

use super::{BootstrapCommandBuilder, ServerSettingsBuilder};
use crate::common::{paths::BLOCK0_BIN, server::Server, startup::db::DbBuilderError};

pub struct ServerBootstrapper {
    settings_builder: ServerSettingsBuilder,
}

impl ServerBootstrapper {
    pub fn new() -> Self {
        let mut settings_builder: ServerSettingsBuilder = Default::default();
        settings_builder
            .with_random_localhost_address()
            .with_block0_path(BLOCK0_BIN.to_string());

        Self { settings_builder }
    }

    pub fn with_localhost_address(&mut self, port: u32) -> &mut Self {
        self.settings_builder.with_localhost_address(port);
        self
    }

    pub fn with_db_path<S: Into<String>>(&mut self, db_url: S) -> &mut Self {
        self.settings_builder.with_db_path(db_url.into());
        self
    }

    pub fn with_block0_path<S: Into<String>>(&mut self, block0_path: S) -> &mut Self {
        self.settings_builder.with_block0_path(block0_path.into());
        self
    }

    pub fn start(&self) -> Result<Server, ServerBootstrapperError> {
        let settings = self.settings_builder.build();

        let mut command_builder: BootstrapCommandBuilder = Default::default();

        let mut command = command_builder
            .address(&settings.address.to_string())
            .db_url(&settings.db_url)
            .block0_path(&settings.block0_path)
            .build();

        let child = command.stdout(Stdio::inherit()).spawn()?;

        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(Server::new(child, settings))
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
    #[error("failed to bootstrap")]
    FailToBootstrap,
    #[error("database builder error")]
    DbBuilderError(#[from] DbBuilderError),
}
