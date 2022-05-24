mod img;
mod payload;

pub use img::KeyQrCode;
pub use payload::{decode, generate};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub const PIN_LENGTH: usize = 4;

#[derive(Debug, PartialEq)]
pub struct QrPin {
    pub password: [u8; 4],
}

#[derive(Error, Debug)]
pub enum BadPinError {
    #[error("The PIN must consist of {PIN_LENGTH} digits, found {0}")]
    InvalidLength(usize),
    #[error("Invalid digit {0}")]
    InvalidDigit(char),
    #[error("cannot detect file name from path {0:?} in order to read qr pin from it")]
    UnableToDetectFileName(PathBuf),
}

impl FromStr for QrPin {
    type Err = BadPinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().count() != PIN_LENGTH {
            return Err(BadPinError::InvalidLength(s.len()));
        }

        let mut pwd = [0u8; 4];
        for (i, digit) in s.chars().enumerate() {
            pwd[i] = digit.to_digit(10).ok_or(BadPinError::InvalidDigit(digit))? as u8;
        }
        Ok(QrPin { password: pwd })
    }
}

#[derive(Clone, Debug)]
pub enum PinReadMode {
    Global(String),
    FromFileName(PathBuf),
}

/// supported format is *1234.png
impl PinReadMode {
    pub fn into_qr_pin(&self) -> Result<QrPin, BadPinError> {
        match self {
            PinReadMode::Global(ref global) => QrPin::from_str(global),
            PinReadMode::FromFileName(qr) => {
                let file_name = qr
                    .file_stem()
                    .ok_or_else(|| BadPinError::UnableToDetectFileName(qr.to_path_buf()))?;
                QrPin::from_str(
                    &file_name
                        .to_str()
                        .unwrap()
                        .chars()
                        .rev()
                        .take(4)
                        .collect::<Vec<char>>()
                        .iter()
                        .rev()
                        .collect::<String>(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_crypto::SecretKey;
    use image::DynamicImage;

    #[test]
    fn parse_pin_successfully() {
        for (pin, pwd) in &[
            ("0000", [0, 0, 0, 0]),
            ("1123", [1, 1, 2, 3]),
            ("0002", [0, 0, 0, 2]),
        ] {
            let qr_pin = QrPin::from_str(pin).unwrap();
            assert_eq!(qr_pin, QrPin { password: *pwd })
        }
    }
    #[test]
    fn pins_that_do_not_satisfy_length_reqs_return_error() {
        for bad_pin in &["", "1", "11", "111", "11111"] {
            let qr_pin = QrPin::from_str(bad_pin);
            assert!(qr_pin.is_err(),)
        }
    }

    #[test]
    fn pins_that_do_not_satisfy_content_reqs_return_error() {
        for bad_pin in &["    ", " 111", "llll", "000u"] {
            let qr_pin = QrPin::from_str(bad_pin);
            assert!(qr_pin.is_err(),)
        }
    }

    // TODO: Improve into an integration test using a temporary directory.
    // Leaving here as an example.
    #[test]
    fn generate_svg() {
        const PASSWORD: &[u8] = &[1, 2, 3, 4];
        let sk = SecretKey::generate(rand::thread_rng());
        let qr = KeyQrCode::generate(sk, PASSWORD);
        qr.write_svg("qr-code.svg").unwrap();
    }

    #[test]
    fn encode_decode() {
        const PASSWORD: &[u8] = &[1, 2, 3, 4];
        let sk = SecretKey::generate(rand::thread_rng());
        let qr = KeyQrCode::generate(sk.clone(), PASSWORD);
        let img = qr.to_img();
        // img.save("qr.png").unwrap();
        assert_eq!(
            sk.leak_secret().as_ref(),
            KeyQrCode::decode(DynamicImage::ImageLuma8(img), PASSWORD).unwrap()[0]
                .clone()
                .leak_secret()
                .as_ref()
        );
    }
}
