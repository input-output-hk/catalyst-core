pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::VitStartParameters;
use crate::mock::config::Configuration;
use crate::mock::mock_state::MockState;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;

pub struct Context {
    config: Configuration,
    address: SocketAddr,
    state: MockState,
}

impl Context {
    pub fn new(config: Configuration, params: VitStartParameters) -> Self {
        Self {
            address: ([0, 0, 0, 0], config.port).into(),
            state: MockState::new(params, config.working_dir.clone()).unwrap(),
            config,
        }
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

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("account does not exists")]
    AccountDoesNotExist,
}

/*
mod date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
*/
