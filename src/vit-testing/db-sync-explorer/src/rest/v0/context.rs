use crate::rest::v0::context::MockOrRealDataProvider::{Mock, Real};
use crate::rest::v0::errors::HandleError;
use crate::rest::v0::DataProvider;
use crate::{Config, MockProvider, Provider};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

pub enum MockOrRealDataProvider {
    Mock(MockProvider),
    Real(Provider),
}

impl MockOrRealDataProvider {
    pub fn as_provider(&self) -> &dyn DataProvider {
        match &self {
            Mock(mock) => mock as &dyn DataProvider,
            Real(real) => real as &dyn DataProvider,
        }
    }
}

pub struct Context {
    data_provider: MockOrRealDataProvider,
    config: Config,
}

impl Context {
    pub fn new_real(real_data_provider: Provider, config: Config) -> Self {
        Self {
            data_provider: Real(real_data_provider),
            config,
        }
    }

    pub fn new_mock(mock_data_provider: MockProvider, config: Config) -> Self {
        Self {
            data_provider: Mock(mock_data_provider),
            config,
        }
    }

    pub fn token(&self) -> &Option<String> {
        &self.config.token
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn provider(&self) -> &dyn DataProvider {
        self.data_provider.as_provider()
    }

    pub fn get_mock_data_provider(&self) -> Result<&MockProvider, HandleError> {
        match &self.data_provider {
            Mock(mock) => Ok(mock),
            Real(_) => Err(HandleError::InternalError(
                "requested operation require db sync to be configured as mock ".to_string(),
            )),
        }
    }

    pub fn get_mock_data_provider_mut(&mut self) -> Result<&mut MockProvider, HandleError> {
        match &mut self.data_provider {
            Mock(mock) => Ok(mock),
            Real(_) => Err(HandleError::InternalError(
                "requested operation require db sync to be configured as mock ".to_string(),
            )),
        }
    }

    pub fn is_mocked(&self) -> bool {
        matches!(self.data_provider, MockOrRealDataProvider::Mock(_))
    }
}

pub fn new_shared_mocked_context(
    mock_data_provider: MockProvider,
    config: Config,
) -> SharedContext {
    let context = Context::new_mock(mock_data_provider, config);
    Arc::new(RwLock::new(context))
}

pub fn new_shared_real_context(data_provider: Provider, config: Config) -> SharedContext {
    let context = Context::new_real(data_provider, config);
    Arc::new(RwLock::new(context))
}
