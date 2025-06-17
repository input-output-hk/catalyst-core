use crate::utils::expiry;
use crate::utils::qr::Error as PinReadError;
use crate::utils::qr::Secret;
use crate::utils::qr::SecretFromQrCode;
use crate::Controller;
use crate::Wallet;
use bech32::FromBase32;
use catalyst_toolbox::kedqr::KeyQrCode;
use catalyst_toolbox::kedqr::PinReadMode;
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;
use jcli_lib::key::read_bech32;
use jormungandr_automation::jormungandr::RestError;
use std::convert::TryInto;
use std::path::Path;
use thiserror::Error;
use valgrind::SettingsExtensions;
use valgrind::ValgrindClient;
use valgrind::ValgrindSettings;
use wallet::Settings;

/// Builder for wallet controller
#[derive(Default)]
pub struct ControllerBuilder {
    backend: Option<ValgrindClient>,
    wallet: Option<Wallet>,
    settings: Option<Settings>,
}

impl ControllerBuilder {
    /// Sets backend from address along with connectivity setup (like cors/https)
    ///
    /// # Errors
    ///
    /// On connectivity issues with given address
    ///
    pub fn with_backend_from_address<S: Into<String>>(
        mut self,
        proxy_address: S,
        backend_settings: ValgrindSettings,
    ) -> Result<Self, Error> {
        let backend = ValgrindClient::new(proxy_address.into(), backend_settings)?;
        self.settings = Some(backend.settings()?.into_wallet_settings());
        self.backend = Some(backend);
        Ok(self)
    }

    /// Sets backend from `ValgrindClient` struct
    ///
    /// # Errors
    ///
    /// On connectivity issues with given address
    ///
    pub fn with_backend_from_client(mut self, backend: ValgrindClient) -> Result<Self, Error> {
        self.settings = Some(backend.settings()?.into_wallet_settings());
        self.backend = Some(backend);
        Ok(self)
    }

    /// Sets wallet based on secret file with bech32 secret key
    ///
    /// # Errors
    ///
    /// On incorrect format of private key (it should be bech32), problem with reading file or
    /// parsing wallet crypto material
    ///
    ///
    pub fn with_wallet_from_secret_file<P: AsRef<Path>>(
        mut self,
        private_key: P,
    ) -> Result<Self, Error> {
        let (_, data, _) = read_bech32(&private_key.as_ref().to_path_buf())?;
        let key_bytes = Vec::<u8>::from_base32(&data)?;
        let data: [u8; 64] = key_bytes
            .try_into()
            .map_err(|_| Error::InvalidSecretKeyLength)?;
        self.wallet = Some(Wallet::recover_from_utxo(&data)?);
        Ok(self)
    }

    /// Sets wallet based on secret key struct
    ///
    /// # Errors
    ///
    /// On incorrect format of private key (it should be bech32)
    /// or parsing wallet crypto material
    ///
    pub fn with_wallet_from_secret_key(
        mut self,
        private_key: SecretKey<Ed25519Extended>,
    ) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover_from_utxo(
            &private_key
                .leak_secret()
                .as_ref()
                .try_into()
                .map_err(|_| Error::CannotLeakSecret)?,
        )?);
        Ok(self)
    }

    /// Sets wallet based on qr code with private key
    ///
    /// # Errors
    ///
    /// On incorrect qr code, problem with reading file or
    /// parsing wallet crypto material
    ///
    /// # Panics
    ///
    /// In unlikely event of 10 not fit in u8 type
    #[allow(clippy::cast_possible_truncation)]
    pub fn with_wallet_from_qr_file<P: AsRef<Path>>(
        mut self,
        qr: P,
        password: &str,
    ) -> Result<Self, Error> {
        let img = image::open(qr.as_ref())?;
        let bytes: Vec<u8> = password
            .chars()
            .map(|x| x.to_digit(10).unwrap() as u8)
            .collect();
        let secret = KeyQrCode::decode(img, &bytes)
            .unwrap().first()
            .ok_or(Error::NoSecretKeyEncoded)?
            .clone()
            .leak_secret();
        self.wallet = Some(Wallet::recover_from_utxo(
            secret
                .as_ref()
                .try_into()
                .map_err(|_| Error::InvalidSecretKeyLength)?,
        )?);
        Ok(self)
    }

    /// Sets wallet based on account bytes
    ///
    /// # Errors
    ///
    /// On parsing wallet crypto material
    ///
    pub fn with_wallet_from_account(mut self, account: &[u8]) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover(account)?);
        Ok(self)
    }

    /// Sets wallet based on payload file (in form of hex encoded string) with private key material
    ///
    /// # Errors
    ///
    /// On reading file or parsing wallet crypto material
    ///
    pub fn with_wallet_from_qr_hash_file<P: AsRef<Path>>(
        mut self,
        qr_payload_file: P,
        password: &str,
    ) -> Result<Self, Error> {
        let pin_read_mode = PinReadMode::Global(password.to_string());

        self.wallet = Some(Wallet::recover(
            Secret::from_payload_file(qr_payload_file.as_ref(), pin_read_mode)?
                .leak_secret()
                .as_ref(),
        )?);
        Ok(self)
    }

    /// Builds Controller
    ///
    /// # Errors
    ///
    /// On any missing configuration steps
    ///
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

/// Builder errors
#[derive(Debug, Error)]
pub enum Error {
    /// Lack of Wallet definition
    #[error("wallet not recovered in builder")]
    WalletNotDefined,
    /// Lack of Backend definition
    #[error("backend not set in builder")]
    BackendNotDefined,
    /// Settings related
    #[error("settings not defined")]
    SettingsNotDefined,
    /// Wallet
    #[error("wallet error")]
    WalletError(#[from] crate::wallet::Error),
    /// Backend
    #[error("backend error")]
    BackendError(#[from] valgrind::Error),
    /// Qr code
    #[error("cannot read QR code from '{0}' path")]
    CannotReadQrCode(#[from] image::ImageError),
    /// Bech32
    #[error("bech32 error")]
    Bech32(#[from] bech32::Error),
    /// Time
    #[error("time error")]
    TimeError(#[from] wallet::time::Error),
    /// Rest
    #[error(transparent)]
    Rest(#[from] RestError),
    /// Reading pin
    #[error(transparent)]
    PinRead(#[from] PinReadError),
    /// Key
    #[error(transparent)]
    Key(#[from] jcli_lib::key::Error),
    /// No secret key exposed
    #[error("no secret key exposed")]
    NoSecretKeyEncoded,
    /// Cannot leak secret
    #[error("cannot leak secret key from qr code structure")]
    CannotLeakSecret,
    /// Cannot leak secret
    #[error("invalid secret key length")]
    InvalidSecretKeyLength,
}
