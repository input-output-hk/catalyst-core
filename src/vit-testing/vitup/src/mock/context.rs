pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::VitStartParameters;
use crate::mock::config::Configuration;
use crate::mock::mock_state::MockState;
use crate::mock::Logger;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use valgrind::VitVersion;

pub struct Context {
    config: Configuration,
    address: SocketAddr,
    state: MockState,
    logger: Logger,
}

impl Context {
    pub fn new(config: Configuration, params: Option<VitStartParameters>) -> Result<Self, Error> {
        Ok(Self {
            address: ([0, 0, 0, 0], config.port).into(),
            state: MockState::new(params.unwrap_or_default(), config.clone())?,
            config,
            logger: Logger::new(),
        })
    }

    pub fn log<S: Into<String>>(&mut self, message: S) {
        self.logger.log(message)
    }

    pub fn logs(&self) -> Vec<String> {
        self.logger.logs()
    }

    pub fn clear_logs(&mut self) {
        self.logger.clear()
    }

    pub fn version(&self) -> VitVersion {
        self.state.version()
    }

    pub fn reset(&mut self, params: VitStartParameters) -> Result<(), Error> {
        self.state = MockState::new(params, self.config.clone())?;
        Ok(())
    }

    pub fn block0_bin(&self) -> Vec<u8> {
        self.state.ledger().block0_bin()
    }

    pub fn working_dir(&self) -> PathBuf {
        self.config.working_dir.clone()
    }

    pub fn available(&self) -> bool {
        self.state.available
    }

    pub fn state(&self) -> &MockState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut MockState {
        &mut self.state
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn api_token(&self) -> Option<String> {
        self.config.token.clone()
    }

    #[allow(dead_code)]
    pub fn set_api_token(&mut self, api_token: String) {
        self.config.token = Some(api_token);
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Ledger(#[from] super::ledger_state::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Mock(#[from] super::mock_state::Error),
}
