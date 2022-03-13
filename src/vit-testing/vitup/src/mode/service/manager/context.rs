pub type ControlContextLock = Arc<Mutex<ControlContext>>;
use super::ServerStopper;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct ControlContext {
    server_stopper: Option<ServerStopper>,
    setup: Config,
    address: SocketAddr,
    working_directory: PathBuf,
    state: State,
    should_stop: bool,
    should_start: bool,
    api_token: Option<String>,
}

impl ControlContext {
    pub fn new<P: AsRef<Path>>(working_dir: P, setup: Config, api_token: Option<String>) -> Self {
        Self {
            server_stopper: None,
            setup,
            working_directory: working_dir.as_ref().to_path_buf(),
            address: ([0, 0, 0, 0], 3030).into(),
            state: State::Idle,
            should_stop: false,
            should_start: false,
            api_token,
        }
    }

    pub fn set_server_stopper(&mut self, server_stopper: ServerStopper) {
        self.server_stopper = Some(server_stopper)
    }

    pub fn set_parameters(&mut self, setup: Config) {
        self.setup = setup;
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        &self.server_stopper
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    pub fn setup(&self) -> &Config {
        &self.setup
    }

    pub fn setup_mut(&mut self) -> &mut Config {
        &mut self.setup
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn request_to_stop(&self) -> bool {
        self.should_stop
    }

    pub fn request_to_start(&self) -> bool {
        self.should_start
    }

    pub fn start(&mut self) {
        self.should_start = true;
    }

    pub fn stop(&mut self) {
        self.should_stop = true;
    }

    pub fn api_token(&self) -> Option<String> {
        self.api_token.clone()
    }

    pub fn api_token_mut(&mut self, api_token: String) {
        self.api_token = Some(api_token);
    }

    pub fn clear_requests(&mut self) {
        self.should_start = false;
        self.should_stop = false;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    Idle,
    Stopping,
    Starting,
    Running,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
