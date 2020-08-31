use super::{BootstrapCommandBuilder, ServerSettingsBuilder};
use crate::common::{paths::BLOCK0_BIN, server::Server, startup::db::DbBuilderError};
use std::process::Stdio;
use thiserror::Error;

pub struct ServerBootstrapper {
    settings_builder: ServerSettingsBuilder,
    allowed_origins: Option<String>,
}

impl ServerBootstrapper {
    pub fn new() -> Self {
        let mut settings_builder: ServerSettingsBuilder = Default::default();
        settings_builder
            .with_random_localhost_address()
            .with_block0_path(BLOCK0_BIN.to_string());

        Self {
            settings_builder,
            allowed_origins: None,
        }
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

    pub fn with_allowed_origins<S: Into<String>>(&mut self, allowed_origins: S) -> &mut Self {
        self.allowed_origins = Some(allowed_origins.into());
        self
    }

    pub fn with_api_tokens(&mut self, enabled: bool) -> &mut Self {
        self.settings_builder.with_api_tokens(enabled);
        self
    }

    pub fn start(&self) -> Result<Server, ServerBootstrapperError> {
        let settings = self.settings_builder.build();

        let mut command_builder: BootstrapCommandBuilder = Default::default();

        command_builder
            .address(&settings.address.to_string())
            .db_url(&settings.db_url)
            .enable_api_tokens(settings.enable_api_tokens)
            .block0_path(&settings.block0_path);

        if let Some(allowed_origins) = self.allowed_origins.as_ref() {
            command_builder.allowed_origins(allowed_origins);
        }
        let mut command = command_builder.build();
        println!("{:?}", command);
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
