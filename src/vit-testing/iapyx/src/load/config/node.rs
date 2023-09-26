use crate::load::MultiControllerError;
use crate::utils::qr::PinReadModeSettings;
use crate::MultiController;
use jormungandr_automation::jormungandr::RestSettings;
use jortestkit::load::Configuration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Node Load configuration struct. It defines all aspects of load run.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Inner configuration which controls common settings like grace period or step delay
    pub config: Configuration,
    /// Use REST API V1, which allows send batch votes or checking individual vote status
    pub use_v1: bool,
    /// Batch size for single load step
    pub batch_size: usize,
    /// Success criteria for run. It helps to put simple assertion on outcome of load scenario.
    /// Usually it should be number from 0 to 100 describing for instance expected success rate for
    /// load messages
    pub criterion: Option<u8>,
    /// REST API address to be excercised by load tool
    pub address: String,
    /// Print verbose information during load
    pub debug: bool,
    /// Use https protocol
    pub use_https: bool,
    /// This parameter work in conjunction with `global_pin` and `qr_codes_folder`.
    /// If source of account secrets is a specific folder with qr codes, then each qr code need to be
    /// individually decrypted using pin code. This parameter defines strategy of acquiring pin for qr code.
    /// Using this parameter and with correct format of file (which should be for example alice_1234.png)
    /// load tool can successfully decrypt all qr code. Alternative for this setting is global pin.
    pub read_pin_from_filename: bool,
    /// Sometimes we may want to run load tool again on the same environment. The problem is that,
    /// Jormungandr blockchain uses account based model so we need to refresh accounts spending counters
    /// This particular settings controls when load tool will refresh account. If set to true load tool
    /// will refresh all accounts before starting the load. It is important because it may be long operation
    /// depending on amount of source accounts
    pub reuse_accounts_early: bool,
    /// This particular settings controls when load tool will refresh account. If set to true load tool
    /// will refresh all accounts just before running votes from particular account, removing necessity to
    /// wait for sync completion like when using `reuse_accounts_early` parameter.
    pub reuse_accounts_lazy: bool,
    /// This parameter work in conjunction with `qr_codes_folder`. If for some reasons file name
    /// does not contain pin, then we can use this setting to set global pin for every qr code
    pub global_pin: String,
    /// Source folder for seed qr codes which can be used during load scenario. Source can be either
    /// qr code folder (defined by `qr_codes_folder`)  or secret key as bech32 file (defined by `secrets_folder`)
    pub qr_codes_folder: Option<PathBuf>,
    /// Source folder for seed bech32 accounts file which will be use during load scenario
    pub secrets_folder: Option<PathBuf>,
    /// Voting groups which account belongs to. Existence of this paramters clearly forbids mixed accounts
    /// (direct voters or reps)
    pub voting_group: String,
}

impl Config {
    /// Gets rest settings
    #[must_use]
    pub fn rest_settings(&self) -> RestSettings {
        RestSettings {
            enable_debug: self.debug,
            use_https: self.use_https,
            ..Default::default()
        }
    }

    /// Construct multi controller (for multi accounts handling)
    ///
    /// # Errors
    ///
    /// On error reading qr or secret files
    ///
    /// # Panics
    ///
    /// On internal os error when extracting path from directory for specific qr code
    pub fn build_multi_controller(&self) -> Result<MultiController, Error> {
        if let Some(qr_codes) = &self.qr_codes_folder {
            let qr_codes: Vec<PathBuf> = std::fs::read_dir(qr_codes)
                .map_err(|_| Error::CannotReadQrs(qr_codes.clone()))?
                .map(|x| x.unwrap().path())
                .collect();

            MultiController::recover_from_qrs(
                &self.address,
                &qr_codes,
                &PinReadModeSettings {
                    from_filename: self.read_pin_from_filename,
                    global_pin: self.global_pin.clone(),
                },
                self.rest_settings(),
            )
            .map_err(Into::into)
        } else if let Some(secrets_folder) = &self.secrets_folder {
            let secrets: Vec<PathBuf> = std::fs::read_dir(secrets_folder)
                .unwrap()
                .map(|x| x.unwrap().path())
                .collect();
            MultiController::recover_from_sks(&self.address, &secrets, self.rest_settings())
                .map_err(Into::into)
        } else {
            Err(Error::CannotFindPrivateKeyRoot)
        }
    }
}

/// Builder related error
#[derive(Error, Debug)]
pub enum Error {
    /// No private key root
    #[error("source of private keys not selected")]
    CannotFindPrivateKeyRoot,
    /// No mnemonic file
    #[error("cannot read mnemonics file")]
    CannotReadMnemonicsFile,
    /// Cannot read qr
    #[error("cannot read folder {0:?}")]
    CannotReadQrs(PathBuf),
    /// MultiController setup
    #[error("multicontoller error")]
    MultiController(#[from] MultiControllerError),
}
