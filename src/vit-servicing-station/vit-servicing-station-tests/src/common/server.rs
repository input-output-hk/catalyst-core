use super::clients::RestClient;
use super::logger::Logger;
use std::path::PathBuf;
use std::process::Child;
use vit_servicing_station_lib::server::settings::ServiceSettings;

pub struct Server {
    process: Child,
    settings: ServiceSettings,
    log_file: PathBuf,
}

impl Server {
    pub fn new(process: Child, settings: ServiceSettings, log_file: PathBuf) -> Self {
        Self {
            process,
            settings,
            log_file,
        }
    }

    pub fn rest_client(&self) -> RestClient {
        RestClient::from(&self.settings)
    }

    pub fn settings(&self) -> ServiceSettings {
        self.settings.clone()
    }

    pub fn rest_client_with_token(&self, token: &str) -> RestClient {
        let mut rest_client = self.rest_client();
        rest_client.set_api_token(token.to_string());
        rest_client
    }

    pub fn logger(&self) -> Logger {
        Logger::new(self.log_file.clone())
    }

    pub fn is_token_valid(&self, token: &str) -> bool {
        self.is_up(token)
    }

    pub fn is_up(&self, token: &str) -> bool {
        self.rest_client_with_token(token).health().is_ok()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        self.process.wait().unwrap();
    }
}
