use jormungandr_testing_utils::qr_code::KeyQrCode;
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
    #[error("cannot divie file name  from path {0:?}")]
    UnableToSplitFileName(PathBuf),
}

#[derive(Debug)]
pub struct QrReader {
    pin_read_mode: PinReadMode,
}

impl QrReader {
    pub fn new(pin_read_mode: PinReadMode) -> Self {
        Self { pin_read_mode }
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
