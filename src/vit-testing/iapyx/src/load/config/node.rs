use crate::load::MultiControllerError;
use crate::utils::qr::PinReadModeSettings;
use crate::MultiController;
use jormungandr_automation::jormungandr::RestSettings;
use jortestkit::load::Configuration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub config: Configuration,
    pub use_v1: bool,
    pub batch_size: usize,
    pub criterion: Option<u8>,
    pub address: String,
    pub debug: bool,
    pub use_https: bool,
    pub read_pin_from_filename: bool,
    pub reuse_accounts_early: bool,
    pub reuse_accounts_lazy: bool,
    pub global_pin: String,
    pub qr_codes_folder: Option<PathBuf>,
    pub secrets_folder: Option<PathBuf>,
}

impl Config {
    pub fn rest_settings(&self) -> RestSettings {
        RestSettings {
            enable_debug: self.debug,
            use_https: self.use_https,
            ..Default::default()
        }
    }

    pub fn build_multi_controller(&self) -> Result<MultiController, Error> {
        if let Some(qr_codes) = &self.qr_codes_folder {
            let qr_codes: Vec<PathBuf> = std::fs::read_dir(qr_codes)
                .map_err(|_| Error::CannotReadQrs(qr_codes.to_path_buf()))?
                .into_iter()
                .map(|x| x.unwrap().path())
                .collect();

            MultiController::recover_from_qrs(
                &self.address,
                &qr_codes,
                PinReadModeSettings {
                    from_filename: self.read_pin_from_filename,
                    global_pin: self.global_pin.clone(),
                },
                self.rest_settings(),
            )
            .map_err(Into::into)
        } else if let Some(secrets_folder) = &self.secrets_folder {
            let secrets: Vec<PathBuf> = std::fs::read_dir(secrets_folder)
                .unwrap()
                .into_iter()
                .map(|x| x.unwrap().path())
                .collect();
            MultiController::recover_from_sks(&self.address, &secrets, self.rest_settings())
                .map_err(Into::into)
        } else {
            Err(Error::CannotFindPrivateKeyRoot)
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("source of private keys not selected")]
    CannotFindPrivateKeyRoot,
    #[error("cannot read mnemonics file")]
    CannotReadMnemonicsFile,
    #[error("cannot read folder {0:?}")]
    CannotReadQrs(PathBuf),
    #[error("multicontoller error")]
    MultiController(#[from] MultiControllerError),
}
