use crate::common::get_available_port;
use crate::common::snapshot::SnapshotServiceController;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use snapshot_trigger_service::config::write_config;
use snapshot_trigger_service::config::Configuration;
use std::path::Path;
use std::path::PathBuf;

use super::Error;
use std::process::Command;

pub struct SnapshotServiceStarter {
    configuration: Configuration,
    path_to_bin: PathBuf,
}

impl Default for SnapshotServiceStarter {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            path_to_bin: Path::new("snapshot-trigger-service").to_path_buf(),
        }
    }
}

impl SnapshotServiceStarter {
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = configuration;
        self
    }

    pub fn with_path_to_bin<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path_to_bin = path.as_ref().to_path_buf();
        self
    }

    pub fn start_on_available_port(
        mut self,
        temp_dir: &TempDir,
    ) -> Result<SnapshotServiceController, Error> {
        self.configuration.address_mut().set_port(get_available_port());
        self.start(temp_dir)
    }

    pub fn start(self, temp_dir: &TempDir) -> Result<SnapshotServiceController, Error> {
        let config_file = temp_dir.child("snapshot_trigger_service_config.yaml");
        write_config(self.configuration.clone(), config_file.path())?;
        let mut command = Command::new(self.path_to_bin.clone());
        command.arg("--config").arg(config_file.path());
        println!("Starting snapshot service: {:?}", command);
        let snapshot_service_bootstrap =
            SnapshotServiceController::new(command.spawn()?, self.configuration.clone());
        self.wait_for_bootstrap(snapshot_service_bootstrap)
    }

    fn wait_for_bootstrap(
        self,
        controller: SnapshotServiceController,
    ) -> Result<SnapshotServiceController, Error> {
        let attempts = 5;
        let mut current_attempt = 1;
        loop {
            if current_attempt > attempts {
                return Err(Error::Bootstrap(self.configuration.address().port()));
            }

            if controller.client().is_up() {
                return Ok(controller);
            }

            println!(
                "waiting for snapshot service bootstrap... {}/{}",
                current_attempt, attempts
            );
            std::thread::sleep(std::time::Duration::from_secs(10));
            current_attempt += 1;
        }
    }
}
