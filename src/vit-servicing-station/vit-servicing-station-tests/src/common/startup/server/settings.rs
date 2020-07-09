use crate::common::startup::get_available_port;
use assert_fs::{fixture::PathChild, TempDir};
use std::{net::SocketAddr, path::PathBuf};
use vit_servicing_station_lib::server::settings::{dump_settings_to_file, ServiceSettings};

pub struct ServerSettingsBuilder {
    settings: ServiceSettings,
}

impl Default for ServerSettingsBuilder {
    fn default() -> Self {
        Self {
            settings: Default::default(),
        }
    }
}

impl ServerSettingsBuilder {
    pub fn with_random_localhost_address(&mut self) -> &mut Self {
        self.with_localhost_address(get_available_port());
        self
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

    pub fn build(&self) -> ServiceSettings {
        self.settings.clone()
    }
}

pub fn dump_settings(temp_dir: &TempDir, settings: &ServiceSettings) -> PathBuf {
    let child_path = temp_dir.child("settings.json");
    dump_settings_to_file(child_path.path().to_str().unwrap(), settings).unwrap();
    child_path.path().into()
}
