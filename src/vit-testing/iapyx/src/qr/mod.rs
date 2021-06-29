use bech32::ToBase32;
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use image::io::Reader as ImageReader;
use jormungandr_testing_utils::qr_code::KeyQrCode;
use jormungandr_testing_utils::qr_code::KeyQrCodeError;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum PinReadMode {
    Global(String),
    ReadFromFileName,
}

impl PinReadMode {
    pub fn new(read_from_file_name: bool, global: &str) -> Self {
        if read_from_file_name {
            return Self::ReadFromFileName;
        }
        Self::Global(global.to_string())
    }
}

#[derive(Debug, Error)]
pub enum PinReadError {
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
}

#[derive(Debug)]
pub struct QrReader {
    pin_read_mode: PinReadMode,
}

impl QrReader {
    pub fn new(pin_read_mode: PinReadMode) -> Self {
        Self { pin_read_mode }
    }

    pub fn read_qr<P: AsRef<Path>>(
        &self,
        qr: P,
    ) -> Result<chain_crypto::SecretKey<chain_crypto::Ed25519Extended>, PinReadError> {
        let pin = get_pin(&self.pin_read_mode, &qr)?;
        let pin = pin_to_bytes(&pin);
        let img = image::open(qr.as_ref())?;
        let secret = KeyQrCode::decode(img, &pin)?;
        Ok(secret.first().unwrap().clone())
    }

    pub fn read_qr_from_bytes(
        &self,
        bytes: Vec<u8>,
    ) -> Result<chain_crypto::SecretKey<chain_crypto::Ed25519Extended>, PinReadError> {
        let pin = match self.pin_read_mode {
            PinReadMode::Global(ref global) => global,
            _ => panic!("when reading qr from bytes Global pin read mode should be used"),
        };
        let pin = pin_to_bytes(&pin);
        let img = image::load_from_memory(&bytes)?;
        let secret = KeyQrCode::decode(img, &pin)?;
        Ok(secret.first().unwrap().clone())
    }

    pub fn read_qr_as_bech32<P: AsRef<Path>>(&self, qr: P) -> Result<String, PinReadError> {
        let sk = self.read_qr(qr)?;
        let hrp = Ed25519Extended::SECRET_BECH32_HRP;
        Ok(bech32::encode(hrp, sk.leak_secret().to_base32())?)
    }

    pub fn read_qrs<P: AsRef<Path>>(
        &self,
        qrs: &[P],
        stop_at_fail: bool,
    ) -> Vec<chain_crypto::SecretKey<chain_crypto::Ed25519Extended>> {
        let mut secrets = Vec::new();
        for (idx, qr) in qrs.iter().enumerate() {
            println!("[{}/{}] Decoding {:?}", idx + 1, qrs.len(), qr.as_ref());

            let pin = match get_pin(&self.pin_read_mode, &qr) {
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
            let pin = pin_to_bytes(&pin);
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
                true => Ok(KeyQrCode::decode(img, &pin).unwrap()),
                false => std::panic::catch_unwind(|| KeyQrCode::decode(img, &pin).unwrap()),
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
}

pub fn pin_to_bytes(pin: &str) -> Vec<u8> {
    pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect()
}

pub fn get_pin<P: AsRef<Path>>(pin_read_mode: &PinReadMode, qr: P) -> Result<String, PinReadError> {
    match pin_read_mode {
        PinReadMode::Global(ref global) => Ok(global.to_string()),
        PinReadMode::ReadFromFileName => {
            let file_name = qr
                .as_ref()
                .file_stem()
                .ok_or_else(|| PinReadError::UnableToDetectFileName(qr.as_ref().to_path_buf()))?;
            Ok(file_name
                .to_str()
                .unwrap()
                .chars()
                .rev()
                .take(4)
                .collect::<Vec<char>>()
                .iter()
                .rev()
                .collect())
        }
    }
}
