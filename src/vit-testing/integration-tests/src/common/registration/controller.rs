use registration_service::{client::rest::RegistrationRestClient, config::Configuration};

use crate::common::registration::remote::RemoteRegistrationServiceController;
use assert_fs::TempDir;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::MainnetWallet;
use registration_service::client::RegistrationResult;
use std::process::Child;

pub struct RegistrationServiceController {
    child: Child,
    remote_controller: RemoteRegistrationServiceController,
}

impl RegistrationServiceController {
    pub fn new(child: Child, configuration: Configuration) -> Self {
        Self {
            child,
            remote_controller: RemoteRegistrationServiceController::new(configuration),
        }
    }

    pub fn client(&self) -> &RegistrationRestClient {
        self.remote_controller.client()
    }

    pub fn shutdown(mut self) -> Result<(), std::io::Error> {
        self.child.kill()
    }

    pub fn configuration(&self) -> &Configuration {
        self.remote_controller.configuration()
    }

    pub fn self_register(&self, wallet: &MainnetWallet, temp_dir: &TempDir) -> RegistrationResult {
        self.remote_controller.self_register(wallet, temp_dir)
    }

    pub fn delegated_register(
        &self,
        wallet: &MainnetWallet,
        delegations: Vec<(Identifier, u32)>,
        temp_dir: &TempDir,
    ) -> RegistrationResult {
        self.remote_controller
            .register_with_delegation(wallet, delegations, temp_dir)
    }
}

impl Drop for RegistrationServiceController {
    fn drop(&mut self) {
        println!("shutting down registration service");
        // There's no kill like overkill
        let _ = self.child.kill();
        // FIXME: These should be better done in a test harness
        self.child.wait().unwrap();
    }
}
