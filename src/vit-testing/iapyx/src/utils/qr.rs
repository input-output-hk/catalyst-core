use bech32::ToBase32;
use catalyst_toolbox::kedqr::BadPinError;
use catalyst_toolbox::kedqr::PinReadMode;
use catalyst_toolbox::kedqr::{decode as decode_hash, KeyQrCode, KeyQrCodeError};
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

/// Qr handling Errors
#[derive(Debug, Error)]
pub enum Error {
    /// Wrong pin reading mode
    #[error("when reading qr from bytes Global pin read mode should be used")]
    GlobalModeOnly,
    /// Cannot read pin
    #[error("cannot read pin from filename {0:?}, expected_format: xxxxxx_1234.xxx")]
    CannotReadPinFromFile(PathBuf),
    /// Cannot get filename
    #[error("cannot detect file name from path {0:?}")]
    UnableToDetectFileName(PathBuf),
    /// Cannot read file
    #[error("cannot read file name from path {0:?}")]
    UnableToReadFileName(PathBuf),
    /// Cannot split file name
    #[error("cannot split file name  from path {0:?}")]
    UnableToSplitFileName(PathBuf),
    /// Cannot read qr
    #[error("Cannot read qr from file")]
    UnableToReadQr(#[from] std::io::Error),
    /// Cannot get secret key
    #[error("Cannot get secret key")]
    UnableToGetSecretKey(#[from] bech32::Error),
    /// Cannot decode qr
    #[error("Cannot decode qr from file")]
    UnableToDecodeQr(#[from] KeyQrCodeError),
    /// Cannot open image
    #[error("cannot open image")]
    UnableToOpenImage(#[from] image::ImageError),
    /// Cannot  decode hash
    #[error("cannot decode hash")]
    CannotDecodeHash(String),
    /// Cannot decode qr
    #[error("Cannot decode qr from file")]
    BadPin(#[from] BadPinError),
}

/// Read mode for Pin. It can try to read it from file name or just set arbitrary
pub struct PinReadModeSettings {
    /// Try to read it from filename
    pub from_filename: bool,
    /// Static one
    pub global_pin: String,
}

impl PinReadModeSettings {
    /// Convert to `PinReadMode` based on path
    pub fn into_qr_pin_mode<P: AsRef<Path>>(&self, path: P) -> PinReadMode {
        if self.from_filename {
            PinReadMode::FromFileName(path.as_ref().to_path_buf())
        } else {
            PinReadMode::Global(self.global_pin.clone())
        }
    }
}

/// Alias for secret key
pub type Secret = chain_crypto::SecretKey<Ed25519Extended>;

/// Trait for converting qr code into secret key
pub trait SecretFromQrCode {
    /// Convert to secret from qr code file
    ///
    /// # Errors
    ///
    /// On incorrect format
    fn from_file<P: AsRef<Path>>(qr: P, pin_read_mode: PinReadMode) -> Result<Secret, Error>;

    /// Convert to secret from bytes
    ///
    /// # Errors
    ///
    /// On illegal bytes length
    fn from_bytes(bytes: Vec<u8>, pin_read_mode: PinReadMode) -> Result<Secret, Error>;

    /// Convert secret to bech32 format
    ///
    /// # Errors
    ///
    /// On converting to bech32 error
    fn to_bech32(self) -> Result<String, Error>;

    /// Convert from qr code payload
    ///
    /// # Errors
    ///
    /// Read from payload file
    fn from_payload_file<P: AsRef<Path>>(
        payload_file: P,
        pin_read_mode: PinReadMode,
    ) -> Result<Secret, Error>;
}

impl SecretFromQrCode for Secret {
    fn from_file<P: AsRef<Path>>(qr: P, pin_read_mode: PinReadMode) -> Result<Secret, Error> {
        let img = image::open(qr.as_ref())?;
        let secret = KeyQrCode::decode(img, &pin_read_mode.into_qr_pin()?.password)?;
        Ok(secret.first().unwrap().clone())
    }

    fn from_bytes(bytes: Vec<u8>, pin_read_mode: PinReadMode) -> Result<Secret, Error> {
        let img = image::load_from_memory(&bytes)?;
        let secret = KeyQrCode::decode(img, &pin_read_mode.into_qr_pin()?.password)?;
        Ok(secret.first().unwrap().clone())
    }

    fn to_bech32(self) -> Result<String, Error> {
        Ok(bech32::encode(
            Ed25519Extended::SECRET_BECH32_HRP,
            self.leak_secret().to_base32(),
            bech32::Variant::Bech32,
        )?)
    }

    fn from_payload_file<P: AsRef<Path>>(
        payload_file: P,
        pin_read_mode: PinReadMode,
    ) -> Result<Secret, Error> {
        let pin = pin_read_mode.into_qr_pin()?.password.to_vec();
        let contents = std::fs::read_to_string(payload_file.as_ref())?;
        decode_hash(contents.clone(), &pin).map_err(|_| Error::CannotDecodeHash(contents))
    }
}

/// Reads qr codes from paths
///
/// # Panics
///
/// When stop at fail is true and there is an error when reading qr code
pub fn read_qrs<P: AsRef<Path>>(
    qrs: &[P],
    pin_read_mode: &PinReadModeSettings,
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

        let result = if stop_at_fail {
            Ok(KeyQrCode::decode(img, &pin.password).unwrap())
        } else {
            std::panic::catch_unwind(|| KeyQrCode::decode(img, &pin.password).unwrap())
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
        secrets.push(secret.first().unwrap().clone());
    }
    secrets
}
