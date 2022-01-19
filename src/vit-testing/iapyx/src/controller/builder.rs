use crate::utils::bech32::read_bech32;
use crate::utils::expiry;
use crate::Controller;
use crate::Wallet;
use bech32::FromBase32;
use bip39::Type;
use catalyst_toolbox::kedqr::KeyQrCode;
use jormungandr_automation::jormungandr::RestError;
use std::convert::TryInto;
use std::path::Path;
use thiserror::Error;
use valgrind::SettingsExtensions;
use valgrind::ValgrindClient;
use valgrind::ValgrindSettings;
use wallet::Settings;

pub struct ControllerBuilder {
    backend: Option<ValgrindClient>,
    wallet: Option<Wallet>,
    settings: Option<Settings>,
}

impl Default for ControllerBuilder {
    fn default() -> Self {
        ControllerBuilder {
            backend: None,
            wallet: None,
            settings: None,
        }
    }
}

impl ControllerBuilder {
    pub fn from_address<S: Into<String>>(
        mut self,
        proxy_address: S,
        backend_settings: ValgrindSettings,
    ) -> Result<Self, Error> {
        let backend = ValgrindClient::new(proxy_address.into(), backend_settings);
        self.settings = Some(backend.settings()?.into_wallet_settings());
        self.backend = Some(backend);
        Ok(self)
    }

    pub fn from_client(mut self, backend: ValgrindClient) -> Result<Self, Error> {
        self.settings = Some(backend.settings()?.into_wallet_settings());
        self.backend = Some(backend);
        Ok(self)
    }

    pub fn from_secret_file<P: AsRef<Path>>(mut self, private_key: P) -> Result<Self, Error> {
        let (_, data) = read_bech32(private_key)?;
        let key_bytes = Vec::<u8>::from_base32(&data)?;
        let data: [u8; 64] = key_bytes.try_into().unwrap();
        self.wallet = Some(Wallet::recover_from_utxo(&data)?);
        Ok(self)
    }

    pub fn from_qr<P: AsRef<Path>>(mut self, qr: P, password: &str) -> Result<Self, Error> {
        let img = image::open(qr.as_ref())?;
        let bytes: Vec<u8> = password
            .chars()
            .map(|x| x.to_digit(10).unwrap() as u8)
            .collect();
        let secret = KeyQrCode::decode(img, &bytes)
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
            .leak_secret();
        self.wallet = Some(Wallet::recover_from_utxo(
            secret.as_ref().try_into().unwrap(),
        )?);
        Ok(self)
    }

    pub fn from_account(mut self, account: &[u8]) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover_from_account(account)?);
        Ok(self)
    }

    pub fn from_mnemonics(mut self, mnemonics: &str, password: &[u8]) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover(mnemonics, password)?);
        Ok(self)
    }

    pub fn generate(mut self, words_length: Type) -> Result<Self, Error> {
        self.wallet = Some(Wallet::generate(words_length)?);
        Ok(self)
    }

    pub fn build(self) -> Result<Controller, Error> {
        let backend = self.backend.ok_or(Error::WalletNotDefined)?;

        Ok(Controller {
            backend: backend.clone(),
            wallet: self.wallet.ok_or(Error::BackendNotDefined)?,
            settings: self.settings.ok_or(Error::SettingsNotDefined)?,
            block_date_generator: expiry::default_block_date_generator(
                &backend.node_client().settings()?,
            ),
        })
    }
}
#[derive(Debug, Error)]
pub enum Error {
    #[error("wallet not recovered in builder")]
    WalletNotDefined,
    #[error("backend not set in builder")]
    BackendNotDefined,
    #[error("settings not defined")]
    SettingsNotDefined,
    #[error("wallet error")]
    WalletError(#[from] crate::wallet::Error),
    #[error("backend error")]
    BackendError(#[from] valgrind::Error),
    #[error("cannot read QR code from '{0}' path")]
    CannotReadQrCode(#[from] image::ImageError),
    #[error("bech32 error")]
    Bech32(#[from] bech32::Error),
    #[error("time error")]
    TimeError(#[from] wallet::time::Error),
    #[error(transparent)]
    Rest(#[from] RestError),
}
