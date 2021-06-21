mod config;
mod multi_controller;
mod request_generators;
mod status_provider;

pub use config::IapyxLoadConfig;
use jortestkit::measurement::EfficiencyBenchmarkFinish;
pub use multi_controller::{MultiController, MultiControllerError};
pub use request_generators::{BatchWalletRequestGen, WalletRequestGen};
pub use status_provider::VoteStatusProvider;
use std::path::PathBuf;
use thiserror::Error;

pub struct IapyxLoad {
    config: IapyxLoadConfig,
}

impl IapyxLoad {
    pub fn new(config: IapyxLoadConfig) -> Self {
        Self { config }
    }

    pub fn start(self) -> Result<Option<EfficiencyBenchmarkFinish>, IapyxLoadError> {
        let backend = self.config.address.clone();

        let settings = self.config.rest_settings();
        let pin_read_mode = self.config.pin_read_mode();

        let multicontroller = {
            if let Some(mnemonics_file) = &self.config.wallet_mnemonics_file {
                let mnemonics = jortestkit::file::read_file_as_vector(&mnemonics_file)
                    .map_err(|_e| IapyxLoadError::CannotReadMnemonicsFile)?;

                MultiController::recover(&backend, mnemonics, &[], settings)
            } else if let Some(qr_codes) = &self.config.qr_codes_folder {
                let qr_codes: Vec<PathBuf> = std::fs::read_dir(qr_codes)
                    .map_err(|_| IapyxLoadError::CannotReadQrs(qr_codes.to_path_buf()))?
                    .into_iter()
                    .map(|x| x.unwrap().path())
                    .collect();

                MultiController::recover_from_qrs(&backend, &qr_codes, pin_read_mode, settings)
            } else if let Some(secrets_folder) = &self.config.secrets_folder {
                let secrets: Vec<PathBuf> = std::fs::read_dir(secrets_folder)
                    .unwrap()
                    .into_iter()
                    .map(|x| x.unwrap().path())
                    .collect();
                MultiController::recover_from_sks(&backend, &secrets, settings)
            } else {
                panic!("source of private keys not selected");
            }
        };

        let measurement_name = "iapyx load test";

        let stats = if self.config.batch_size > 1 {
            jortestkit::load::start_async(
                BatchWalletRequestGen::new(
                    multicontroller?,
                    self.config.batch_size,
                    self.config.use_v1,
                ),
                VoteStatusProvider::new(backend, self.config.debug),
                self.config.config,
                measurement_name,
            )
        } else {
            jortestkit::load::start_sync(
                WalletRequestGen::new(multicontroller?),
                self.config.config,
                measurement_name,
            )
        };

        stats.print_summary(measurement_name);

        if let Some(threshold) = self.config.criterion {
            return Ok(Some(stats.measure(measurement_name, threshold.into())));
        }
        Ok(None)
    }
}

#[derive(Error, Debug)]
pub enum IapyxLoadError {
    #[error("cannot read mnemonics file")]
    CannotReadMnemonicsFile,
    #[error("cannot read folder {0:?}")]
    CannotReadQrs(PathBuf),
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
}
