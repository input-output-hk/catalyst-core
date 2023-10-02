use crate::common::startup::get_exe;
use std::path::Path;
use std::{path::PathBuf, process::Command};

/// In order to test robustness of server bootstrapper we need to be able
/// to provide some
pub struct BootstrapCommandBuilder {
    exe: PathBuf,
    address: Option<String>,
    allowed_origins: Option<String>,
    block0_path: Option<String>,
    cert_file: Option<PathBuf>,
    db_url: Option<String>,
    enable_api_tokens: bool,
    in_settings_file: Option<PathBuf>,
    max_age_secs: Option<u32>,
    out_settings_file: Option<PathBuf>,
    priv_key_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
    log_level: Option<String>,
    service_version: Option<String>,
}

impl Default for BootstrapCommandBuilder {
    fn default() -> Self {
        Self::new(get_exe())
    }
}

impl BootstrapCommandBuilder {
    pub fn new(exe: PathBuf) -> Self {
        Self {
            exe,
            address: None,
            allowed_origins: None,
            block0_path: None,
            cert_file: None,
            db_url: None,
            enable_api_tokens: false,
            in_settings_file: None,
            max_age_secs: None,
            out_settings_file: None,
            priv_key_file: None,
            log_file: None,
            log_level: None,
            service_version: None,
        }
    }

    pub fn address<S: Into<String>>(&mut self, address: S) -> &mut Self {
        self.address = Some(address.into());
        self
    }

    pub fn allowed_origins<S: Into<String>>(&mut self, allowed_origins: S) -> &mut Self {
        self.allowed_origins = Some(allowed_origins.into());
        self
    }

    pub fn block0_path<S: Into<String>>(&mut self, block0_path: S) -> &mut Self {
        self.block0_path = Some(block0_path.into());
        self
    }

    pub fn cert_file(&mut self, cert_file: &Path) -> &mut Self {
        self.cert_file = Some(cert_file.to_path_buf());
        self
    }

    pub fn db_url<S: Into<String>>(&mut self, db_url: S) -> &mut Self {
        self.db_url = Some(db_url.into());
        self
    }

    pub fn enable_api_tokens(&mut self, enabled: bool) -> &mut Self {
        self.enable_api_tokens = enabled;
        self
    }

    pub fn in_settings_file(&mut self, in_settings_file: &Path) -> &mut Self {
        self.in_settings_file = Some(in_settings_file.to_path_buf());
        self
    }
    pub fn max_age_secs(&mut self, max_age_secs: u32) -> &mut Self {
        self.max_age_secs = Some(max_age_secs);
        self
    }
    pub fn out_settings_file(&mut self, out_settings_file: &Path) -> &mut Self {
        self.out_settings_file = Some(out_settings_file.to_path_buf());
        self
    }

    pub fn priv_key_file(&mut self, priv_key_file: &Path) -> &mut Self {
        self.priv_key_file = Some(priv_key_file.to_path_buf());
        self
    }

    pub fn log_file(&mut self, log_file: &Path) -> &mut Self {
        self.log_file = Some(log_file.to_path_buf());
        self
    }

    pub fn log_level(&mut self, log_level: &str) -> &mut Self {
        self.log_level = Some(log_level.to_string());
        self
    }

    pub fn service_version<S: Into<String>>(&mut self, service_version: S) -> &mut Self {
        self.service_version = Some(service_version.into());
        self
    }

    pub fn build(&self) -> Command {
        let mut command = Command::new(self.exe.clone());

        let service_version = if let Some(service_version) = &self.service_version {
            service_version.clone()
        } else {
            Default::default()
        };
        command.arg("--service-version").arg(service_version);

        if let Some(address) = &self.address {
            command.arg("--address").arg(address);
        }

        if let Some(allowed_origins) = &self.allowed_origins {
            command.arg("--allowed-origins").arg(allowed_origins);
        }

        if let Some(block0_path) = &self.block0_path {
            command.arg("--block0-path").arg(block0_path);
        }

        if let Some(cert_file) = &self.cert_file {
            command.arg("--cert-file").arg(cert_file.to_str().unwrap());
        }

        if let Some(db_url) = &self.db_url {
            command.arg("--db-url").arg(db_url);
        }

        if let Some(in_settings_file) = &self.in_settings_file {
            command
                .arg("--in-settings-file")
                .arg(in_settings_file.to_str().unwrap());
        }

        if let Some(max_age_secs) = &self.max_age_secs {
            command.arg("--max-age-secs").arg(max_age_secs.to_string());
        }

        if let Some(out_settings_file) = &self.out_settings_file {
            command
                .arg("--out-settings-file")
                .arg(out_settings_file.to_str().unwrap());
        }

        if let Some(priv_key_file) = &self.priv_key_file {
            command
                .arg("--priv-key-file")
                .arg(priv_key_file.to_str().unwrap());
        }

        if self.enable_api_tokens {
            command.arg("--enable-api-tokens");
        }

        if let Some(log_file) = &self.log_file {
            command
                .arg("--log-output-path")
                .arg(log_file.to_str().unwrap());
        }

        if let Some(log_level) = &self.log_level {
            command.arg("--log-level").arg(log_level);
        }

        command
    }
}
