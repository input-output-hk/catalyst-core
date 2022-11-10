use crate::common::get_available_port;
use crate::common::registration::RegistrationServiceController;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use registration_service::config::{read_config, write_config};
use std::path::Path;
use std::path::PathBuf;

use super::Error;
use crate::common::registration::remote::RemoteRegistrationServiceController;
use registration_service::config;
use registration_service::config::Configuration;
use std::process::Command;

pub struct RegistrationServiceStarter {
    configuration: Configuration,
    path_to_bin: PathBuf,
}

impl Default for RegistrationServiceStarter {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            path_to_bin: Path::new("registration-service").to_path_buf(),
        }
    }
}

impl RegistrationServiceStarter {
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = configuration;
        self
    }

    pub fn with_configuration_from<P: AsRef<Path>>(
        mut self,
        config: P,
    ) -> Result<Self, config::Error> {
        self.configuration = read_config(config)?;
        Ok(self)
    }

    pub fn with_path_to_bin<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path_to_bin = path.as_ref().to_path_buf();
        self
    }

    pub fn start_on_available_port(
        mut self,
        temp_dir: &TempDir,
    ) -> Result<RegistrationServiceController, Error> {
        let address = self.configuration.address_mut();
        address.set_port(get_available_port());
        self.start(temp_dir)
    }

    pub fn attach_to(self) -> RemoteRegistrationServiceController {
        RemoteRegistrationServiceController::new(self.configuration)
    }

    pub fn start(self, temp_dir: &TempDir) -> Result<RegistrationServiceController, Error> {
        let config_file = temp_dir.child("registration_service_config.yaml");
        write_config(self.configuration.clone(), config_file.path())?;
        let mut command = Command::new(self.path_to_bin.clone());
        command.arg("--config").arg(config_file.path());

        println!("Starting registration service: {:?}", command);

        let registration_service_bootstrap =
            RegistrationServiceController::new(command.spawn()?, self.configuration.clone());
        self.wait_for_bootstrap(registration_service_bootstrap)
    }

    fn wait_for_bootstrap(
        self,
        controller: RegistrationServiceController,
    ) -> Result<RegistrationServiceController, Error> {
        let attempts = 500;
        let mut current_attempt = 1;
        loop {
            if current_attempt > attempts {
                return Err(Error::Bootstrap(self.configuration.address().port()));
            }

            if controller.client().is_up() {
                return Ok(controller);
            }

            println!(
                "waiting for registration service bootstrap... {}/{}",
                current_attempt, attempts
            );
            std::thread::sleep(std::time::Duration::from_secs(10));
            current_attempt += 1;
        }
    }
}
