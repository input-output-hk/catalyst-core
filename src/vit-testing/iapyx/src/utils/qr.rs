use bech32::ToBase32;
use catalyst_toolbox::kedqr::BadPinError;
use catalyst_toolbox::kedqr::PinReadMode;
use catalyst_toolbox::kedqr::{decode as decode_hash, KeyQrCode, KeyQrCodeError};
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PinReadError {
    #[error("when reading qr from bytes Global pin read mode should be used")]
    GlobalModeOnly,
    #[error("cannot read pin from filename {0:?}, expected_format: xxxxxx_1234.xxx")]
    CannotReadPinFromFile(PathBuf),
    #[error("cannot detect file name from path {0:?}")]
    UnableToDetectFileName(PathBuf),
    #[error("cannot read file name from path {0:?}")]
    UnableToReadFileName(PathBuf),
    #[error("cannot split file name  from path {0:?}")]
    UnableToSplitFileName(PathBuf),
    #[error("Cannot read qr from file")]
    UnableToReadQr(#[from] std::io::Error),
    #[error("Cannot get secret key")]
    UnableToGetSecretKey(#[from] bech32::Error),
    #[error("Cannot decode qr from file")]
    UnableToDecodeQr(#[from] KeyQrCodeError),
    #[error("cannot open image")]
    UnableToOpenImage(#[from] image::ImageError),
    #[error("cannot decode hash")]
    CannotDecodeHash(String),
    #[error("Cannot decode qr from file")]
    BadPin(#[from] BadPinError),
}

pub struct PinReadModeSettings {
    pub from_filename: bool,
    pub global_pin: String,
}

impl PinReadModeSettings {
    pub fn into_qr_pin_mode<P: AsRef<Path>>(&self, path: P) -> PinReadMode {
        if self.from_filename {
            PinReadMode::FromFileName(path.as_ref().to_path_buf())
        } else {
            PinReadMode::Global(self.global_pin.clone())
        }
    }
}

pub type Secret = chain_crypto::SecretKey<Ed25519Extended>;

pub trait SecretFromQrCode {
    fn from_file<P: AsRef<Path>>(qr: P, pin_read_mode: PinReadMode)
        -> Result<Secret, PinReadError>;

    fn from_bytes(bytes: Vec<u8>, pin_read_mode: PinReadMode) -> Result<Secret, PinReadError>;

    fn to_bech32(self) -> Result<String, PinReadError>;

    fn from_payload_file<P: AsRef<Path>>(
        payload_file: P,
        pin_read_mode: PinReadMode,
    ) -> Result<Secret, PinReadError>;
}

impl SecretFromQrCode for Secret {
    fn from_file<P: AsRef<Path>>(
        qr: P,
        pin_read_mode: PinReadMode,
    ) -> Result<Secret, PinReadError> {
        let img = image::open(qr.as_ref())?;
        let secret = KeyQrCode::decode(img, &pin_read_mode.into_qr_pin()?.password)?;
        Ok(secret.first().unwrap().clone())
    }

    fn from_bytes(bytes: Vec<u8>, pin_read_mode: PinReadMode) -> Result<Secret, PinReadError> {
        let img = image::load_from_memory(&bytes)?;
        let secret = KeyQrCode::decode(img, &pin_read_mode.into_qr_pin()?.password)?;
        Ok(secret.first().unwrap().clone())
    }

    fn to_bech32(self) -> Result<String, PinReadError> {
        Ok(bech32::encode(
            Ed25519Extended::SECRET_BECH32_HRP,
            self.leak_secret().to_base32(),
            bech32::Variant::Bech32,
        )?)
    }

    fn from_payload_file<P: AsRef<Path>>(
        payload_file: P,
        pin_read_mode: PinReadMode,
    ) -> Result<Secret, PinReadError> {
        let pin = pin_read_mode.into_qr_pin()?.password.to_vec();
        let contents = std::fs::read_to_string(payload_file.as_ref())?;
        decode_hash(contents.clone(), &pin).map_err(|_| PinReadError::CannotDecodeHash(contents))
    }
}

pub fn read_qrs<P: AsRef<Path>>(
    qrs: &[P],
    pin_read_mode: PinReadModeSettings,
    stop_at_fail: bool,
) -> Vec<Secret> {
    let mut secrets = Vec::new();
    for (idx, qr) in qrs.iter().enumerate() {
        println!("[{}/{}] Decoding {:?}", idx + 1, qrs.len(), qr.as_ref());

        let pin = match pin_read_mode.into_qr_pin_mode(qr).into_qr_pin() {
            Ok(pin) => pin,
            Err(err) => {
                println!(
                    "Cannot detect pin from file: {:?}, due to {:?}",
                    qr.as_ref(),
                    err
                );
                continue;
            }
        };
        let img = match image::open(qr.as_ref()) {
            Ok(img) => img,
            Err(err) => {
                println!(
                    "Cannot read qr from file: {:?}, due to {:?}",
                    qr.as_ref(),
                    err
                );
                continue;
            }
        };

        let result = match stop_at_fail {
            true => Ok(KeyQrCode::decode(img, &pin.password).unwrap()),
            false => std::panic::catch_unwind(|| KeyQrCode::decode(img, &pin.password).unwrap()),
        };
        let secret = match result {
            Ok(secret) => secret,
            Err(err) => {
                println!(
                    "Cannot decode qr from file: {:?}, due to {:?}",
                    qr.as_ref(),
                    err
                );
                continue;
            }
        };
        secrets.push(secret.get(0).unwrap().clone());
    }
    secrets
}
