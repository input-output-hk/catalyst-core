use crate::config::Configuration;
use crate::ServerStopper;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<SchedulerContext>>;

#[derive(Clone)]
pub struct SchedulerContext {
    server_stopper: Option<ServerStopper>,
    config: Configuration,
}

impl SchedulerContext {
    pub fn new(server_stopper: Option<ServerStopper>, config: Configuration) -> Self {
        Self {
            server_stopper,
            config,
        }
    }

    pub fn admin_token(&self) -> Option<String> {
        self.config.admin_token.clone()
    }
    pub fn api_token(&self) -> Option<String> {
        self.config.api_token.clone()
    }

    pub fn set_admin_token(&mut self, admin_token: Option<String>) {
        self.config.admin_token = admin_token;
    }
    pub fn set_api_token(&mut self, api_token: Option<String>) {
        self.config.api_token = api_token;
    }

    pub fn working_directory(&self) -> &Path {
        &self.config.result_dir
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        &self.server_stopper
    }
    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn set_server_stopper(&mut self, server_stopper: Option<ServerStopper>) {
        self.server_stopper = server_stopper;
    }
}
