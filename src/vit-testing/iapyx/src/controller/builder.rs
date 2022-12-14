use crate::utils::expiry;
use crate::utils::qr::PinReadError;
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

#[derive(Default)]
pub struct ControllerBuilder {
    backend: Option<ValgrindClient>,
    wallet: Option<Wallet>,
    settings: Option<Settings>,
}

impl ControllerBuilder {
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

    pub fn with_backend_from_client(mut self, backend: ValgrindClient) -> Result<Self, Error> {
        self.settings = Some(backend.settings()?.into_wallet_settings());
        self.backend = Some(backend);
        Ok(self)
    }

    pub fn with_wallet_from_secret_file<P: AsRef<Path>>(
        mut self,
        private_key: P,
    ) -> Result<Self, Error> {
        let (_, data, _) = read_bech32(&private_key.as_ref().to_path_buf())?;
        let key_bytes = Vec::<u8>::from_base32(&data)?;
        let data: [u8; 64] = key_bytes.try_into().unwrap();
        self.wallet = Some(Wallet::recover_from_utxo(&data)?);
        Ok(self)
    }

    pub fn with_wallet_from_secret_key(
        mut self,
        private_key: SecretKey<Ed25519Extended>,
    ) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover_from_utxo(
            &private_key.leak_secret().as_ref().try_into().unwrap(),
        )?);
        Ok(self)
    }

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

    pub fn with_wallet_from_account(mut self, account: &[u8]) -> Result<Self, Error> {
        self.wallet = Some(Wallet::recover(account)?);
        Ok(self)
    }

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
    #[error(transparent)]
    PinRead(#[from] PinReadError),
    #[error(transparent)]
    Key(#[from] jcli_lib::key::Error),
}
