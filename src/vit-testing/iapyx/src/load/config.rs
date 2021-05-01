use crate::PinReadMode;
use jormungandr_testing_utils::testing::node::RestSettings;
use jortestkit::load::Configuration;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct IapyxLoadConfig {
    pub config: Configuration,
    pub use_v1: bool,
    pub batch_size: usize,
    pub criterion: Option<u8>,
    pub address: String,
    pub debug: bool,
    pub use_https_for_post: bool,
    pub read_pin_from_filename: bool,
    pub reuse_accounts: bool,
    pub global_pin: String,
    pub wallet_mnemonics_file: Option<PathBuf>,
    pub qr_codes_folder: Option<PathBuf>,
    pub secrets_folder: Option<PathBuf>,
}

impl IapyxLoadConfig {
    pub fn rest_settings(&self) -> RestSettings {
        RestSettings {
            enable_debug: self.debug,
            use_https_for_post: self.use_https_for_post,
            ..Default::default()
        }
    }

    pub fn pin_read_mode(&self) -> PinReadMode {
        PinReadMode::new(self.read_pin_from_filename, &self.global_pin)
    }
}
