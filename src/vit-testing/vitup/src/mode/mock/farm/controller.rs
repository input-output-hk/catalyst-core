use crate::client::rest::VitupDisruptionRestClient;
use crate::client::rest::VitupRest;
use crate::config::CertificatesBuilder;
use crate::mode::mock::config::write_config;
use crate::mode::mock::farm::context::MockId;
use crate::mode::mock::Configuration;
use lazy_static::lazy_static;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags};
use reqwest::Url;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU16, Ordering},
};
use thiserror::Error;

lazy_static! {
    static ref NEXT_AVAILABLE_PORT_NUMBER: AtomicU16 = AtomicU16::new(10000);
    static ref OCCUPIED_PORTS: HashSet<u16> = {
        let af_flags = AddressFamilyFlags::IPV4;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        get_sockets_info(af_flags, proto_flags)
            .unwrap()
            .into_iter()
            .map(|s| s.local_port())
            .collect::<HashSet<_>>()
    };
}

fn get_available_port() -> u16 {
    loop {
        let candidate_port = NEXT_AVAILABLE_PORT_NUMBER.fetch_add(1, Ordering::SeqCst);
        if !(*OCCUPIED_PORTS).contains(&candidate_port) {
            return candidate_port;
        }
    }
}

pub struct MockBootstrap {
    mock_id: MockId,
    configuration: Configuration,
    working_directory: PathBuf,
    https: bool,
}

impl MockBootstrap {
    pub fn new(mock_id: MockId) -> Self {
        Self {
            mock_id,
            configuration: Configuration {
                port: get_available_port(),
                token: None,
                working_dir: PathBuf::new(),
                protocol: valgrind::Protocol::Http,
                local: false,
            },
            https: true,
            working_directory: PathBuf::new(),
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.configuration.port = port;
        self
    }

    pub fn working_directory<P: AsRef<Path>>(mut self, working_directory: P) -> Self {
        self.working_directory = working_directory.as_ref().to_path_buf();
        self
    }

    pub fn https(mut self) -> Self {
        self.https = true;
        self
    }

    pub fn spawn(mut self) -> Result<MockController, Error> {
        if self.https {
            let certs = CertificatesBuilder::default().build(&self.working_directory)?;
            self.configuration.protocol = certs.into();
        }

        self.configuration.working_dir = self.working_directory.join(&self.mock_id).join("data");
        let mut config_path = self.working_directory.join(&self.mock_id);
        std::fs::create_dir_all(&config_path)?;
        config_path = config_path.join("config.yaml");
        write_config(&self.configuration, &config_path)?;

        let mut command = Command::new("vitup");
        command
            .arg("start")
            .arg("mock")
            .arg("--config")
            .arg(config_path);

        Ok(MockController {
            mock_id: self.mock_id,
            configuration: self.configuration,
            process: command.spawn()?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct MockController {
    mock_id: MockId,
    configuration: Configuration,
    #[serde(skip_serializing)]
    process: Child,
}

impl MockController {
    pub fn port(&self) -> u16 {
        self.configuration.port
    }

    pub fn is_up(&self) -> bool {
        let rest_client = {
            if let Some(token) = &self.configuration.token {
                VitupRest::new_with_token(token.clone(), self.address())
            } else {
                VitupRest::new(self.address())
            }
        };
        VitupDisruptionRestClient::from(rest_client).is_up()
    }

    pub fn address(&self) -> String {
        let mut url = Url::parse("http://127.0.0.1").unwrap();
        url.set_scheme(&self.configuration.protocol.schema())
            .unwrap();
        url.set_port(Some(self.configuration.port)).unwrap();
        url.to_string()
    }

    pub fn shutdown(&mut self) {
        let _ = self.process.kill();
    }
}

impl Drop for MockController {
    fn drop(&mut self) {
        self.shutdown();
        self.process.wait().unwrap();
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Certs(#[from] crate::config::certs::Error),
    #[error(transparent)]
    Config(#[from] crate::mode::mock::config::Error),
}
