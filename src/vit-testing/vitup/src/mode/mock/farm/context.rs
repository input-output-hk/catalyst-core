pub type ContextLock = Arc<Mutex<Context>>;
use super::config::Config;
use super::MockBootstrap;
use super::MockController;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use valgrind::Protocol;

pub type MockId = String;
pub type MockState = HashMap<MockId, MockController>;

pub struct Context {
    config: Config,
    state: MockState,
    address: SocketAddr,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            address: if config.local {
                ([127, 0, 0, 1], config.port).into()
            } else {
                ([0, 0, 0, 0], config.port).into()
            },
            state: HashMap::new(),
            config,
        }
    }

    pub fn protocol(&self) -> Protocol {
        self.config.protocol.clone()
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn working_dir(&self) -> PathBuf {
        self.config.working_directory.clone()
    }

    pub fn state_mut(&mut self) -> &mut MockState {
        &mut self.state
    }

    pub fn api_token(&self) -> Option<String> {
        self.config.token.clone()
    }

    #[allow(dead_code)]
    pub fn set_api_token(&mut self, api_token: String) {
        self.config.token = Some(api_token);
    }

    pub fn get_active_mocks(&self) -> HashMap<MockId, u16> {
        self.state
            .iter()
            .map(|(id, controller)| (id.clone(), controller.port()))
            .collect()
    }

    pub fn shutdown_mock(&mut self, id: MockId) -> Result<u16, Error> {
        let mut controller = self.state.remove(&id).ok_or(Error::CannotFindMock(id))?;
        let port = controller.port();
        controller.shutdown();
        Ok(port)
    }

    pub fn start_mock_on_random_port(&mut self, id: MockId) -> Result<u16, Error> {
        if self.state.contains_key(&id) {
            return Err(Error::EnvironmentAlreadyExist(id));
        }

        let mock_controller = MockBootstrap::new(id.clone())
            .https()
            .working_directory(self.config.working_directory.clone())
            .spawn()?;
        let port = mock_controller.port();
        self.state.insert(id, mock_controller);
        Ok(port)
    }

    pub fn start_mock(&mut self, id: MockId, port: u16) -> Result<u16, Error> {
        let mock_controller = MockBootstrap::new(id.clone())
            .port(port)
            .https()
            .working_directory(self.config.working_directory.clone())
            .spawn()?;
        let port = mock_controller.port();
        self.state.insert(id, mock_controller);
        Ok(port)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Controller(#[from] super::ControllerError),
    #[error("cannot find mock env with id: {0}")]
    CannotFindMock(MockId),
    #[error("mock env with name: '{0}' already exist, please choose another name")]
    EnvironmentAlreadyExist(MockId),
}
