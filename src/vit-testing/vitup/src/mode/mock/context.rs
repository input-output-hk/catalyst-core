pub type ContextLock = Arc<RwLock<Context>>;
use super::{mock_state::MockState, Configuration};
use crate::config::Config;
use crate::mode::mock::rest::reject::ForcedErrorCode;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tracing::info;
use valgrind::Protocol;
use valgrind::VitVersion;

pub struct Context {
    config: Configuration,
    address: SocketAddr,
    state: MockState,
}

impl Context {
    pub fn new(config: Configuration, params: Option<Config>) -> Result<Self, Error> {
        Ok(Self {
            address: if config.local {
                ([127, 0, 0, 1], config.port).into()
            } else {
                ([0, 0, 0, 0], config.port).into()
            },
            state: MockState::new(params.unwrap_or_default(), config.clone())?,
            config,
        })
    }

    pub fn version(&self) -> VitVersion {
        self.state.version()
    }

    pub fn reset(&mut self, params: Config) -> Result<(), Error> {
        *self = Self::new(self.config.clone(), Some(params))?;
        Ok(())
    }

    pub fn block0_bin(&self) -> Vec<u8> {
        self.state.block0_bin()
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

    pub fn protocol(&self) -> Protocol {
        self.config.protocol.clone()
    }

    #[allow(dead_code)]
    pub fn set_api_token(&mut self, api_token: String) {
        self.config.token = Some(api_token);
    }

    #[tracing::instrument(skip(self))]
    pub fn check_if_rest_available(&self) -> Option<ForcedErrorCode> {
        if !self.available() {
            let code = self.state().error_code;
            info!(
                "unavailability mode is on. Rejecting any further calls with error code: {}",
                code
            );
            Some(ForcedErrorCode { code })
        } else {
            None
        }
    }
}

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    #[error(transparent)]
    Ledger(#[from] super::ledger_state::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Mock(#[from] super::mock_state::Error),
}
