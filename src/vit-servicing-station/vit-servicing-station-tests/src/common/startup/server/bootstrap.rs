use super::{BootstrapCommandBuilder, ServerSettingsBuilder};
use crate::common::{
    paths::BLOCK0_BIN,
    server::Server,
    startup::{db::DbBuilderError, get_exe},
};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use vit_servicing_station_lib::server::settings::LogLevel;

pub struct ServerBootstrapper {
    settings_builder: ServerSettingsBuilder,
    allowed_origins: Option<String>,
    service_version: String,
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
            service_version: Default::default(),
        }
    }

    pub fn with_localhost_address(&mut self, port: u32) -> &mut Self {
        self.settings_builder.with_localhost_address(port);
        self
    }

    pub fn with_log_level(&mut self, log_level: LogLevel) -> &mut Self {
        self.settings_builder.with_log_level(log_level);
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

    pub fn with_service_version(&mut self, service_version: String) -> &mut Self {
        self.service_version = service_version;
        self
    }

    pub fn start_with_exe(
        &self,
        temp_dir: &TempDir,
        exe: PathBuf,
    ) -> Result<Server, ServerBootstrapperError> {
        let settings = self.settings_builder.build();
        let logger_file: PathBuf = temp_dir.child("log.log").path().into();
        let mut command_builder = BootstrapCommandBuilder::new(exe);

        command_builder
            .address(&settings.address.to_string())
            .db_url(&settings.db_url)
            .log_file(&logger_file)
            .enable_api_tokens(settings.enable_api_tokens)
            .block0_path(&settings.block0_path)
            .service_version(&self.service_version);

        if let Some(allowed_origins) = self.allowed_origins.as_ref() {
            command_builder.allowed_origins(allowed_origins);
        }

        if let Some(log_level) = &settings.log.log_level {
            command_builder.log_level(&serde_json::to_string(&log_level).unwrap());
        }

        let mut command = command_builder.build();
        println!("{:?}", command);
        let child = command.stdout(Stdio::inherit()).spawn()?;

        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(Server::new(child, settings, logger_file))
    }

    pub fn start(&self, temp_dir: &TempDir) -> Result<Server, ServerBootstrapperError> {
        self.start_with_exe(temp_dir, get_exe())
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
