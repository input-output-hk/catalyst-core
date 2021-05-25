use chain_crypto::{Ed25519Extended, SecretKey, SecretKeyError};
use image::{DynamicImage, ImageBuffer, Luma};
use qrcode::{
    render::{svg, unicode},
    EcLevel, QrCode,
};
use std::fmt;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use std::str::FromStr;
use symmetric_cipher::{decrypt, encrypt, Error as SymmetricCipherError};
use thiserror::Error;

pub const PIN_LENGTH: usize = 4;

pub struct KeyQrCode {
    inner: QrCode,
}

#[derive(Error, Debug)]
pub enum KeyQrCodeError {
    #[error("encryption-decryption protocol error")]
    SymmetricCipher(#[from] SymmetricCipherError),
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("invalid secret key")]
    SecretKey(#[from] SecretKeyError),
    #[error("couldn't decode QR code")]
    QrDecodeError(#[from] QrDecodeError),
    #[error("failed to decode hex")]
    HexDecodeError(#[from] hex::FromHexError),
}

#[derive(Error, Debug)]
pub enum QrDecodeError {
    #[error("couldn't decode QR code")]
    DecodeError(#[from] quircs::DecodeError),
    #[error("couldn't extract QR code")]
    ExtractError(#[from] quircs::ExtractError),
    #[error("QR code payload is not valid uf8")]
    NonUtf8Payload,
}

impl KeyQrCode {
    pub fn generate(key: SecretKey<Ed25519Extended>, password: &[u8]) -> Self {
        let secret = key.leak_secret();
        let rng = rand::thread_rng();
        // this won't fail because we already know it's an ed25519extended key,
        // so it is safe to unwrap
        let enc = encrypt(password, secret.as_ref(), rng).unwrap();
        // Using binary would make the QR codes more compact and probably less
        // prone to scanning errors.
        let enc_hex = hex::encode(enc);
        let inner = QrCode::with_error_correction_level(&enc_hex, EcLevel::H).unwrap();

        KeyQrCode { inner }
    }

    pub fn write_svg(&self, path: impl AsRef<Path>) -> Result<(), KeyQrCodeError> {
        let mut out = File::create(path)?;
        let svg_file = self
            .inner
            .render()
            .quiet_zone(true)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();
        out.write_all(svg_file.as_bytes())?;
        out.flush()?;
        Ok(())
    }

    pub fn to_img(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let qr = &self.inner;
        let img = qr.render::<Luma<u8>>().build();
        img
    }

    pub fn decode(
        img: DynamicImage,
        password: &[u8],
    ) -> Result<Vec<SecretKey<Ed25519Extended>>, KeyQrCodeError> {
        let mut decoder = quircs::Quirc::default();

        let img = img.into_luma8();

        let codes = decoder.identify(img.width() as usize, img.height() as usize, &img);

        codes
            .map(|code| -> Result<_, KeyQrCodeError> {
                let decoded = code
                    .map_err(QrDecodeError::ExtractError)
                    .and_then(|c| c.decode().map_err(QrDecodeError::DecodeError))?;

                // TODO: I actually don't know if this can fail
                let h = std::str::from_utf8(&decoded.payload)
                    .map_err(|_| QrDecodeError::NonUtf8Payload)?;
                let encrypted_bytes = hex::decode(h)?;
                let key = decrypt(password, &encrypted_bytes)?;
                Ok(SecretKey::from_binary(&key)?)
            })
            .collect()
    }
}

impl fmt::Display for KeyQrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qr_img = self
            .inner
            .render::<unicode::Dense1x2>()
            .quiet_zone(true)
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        write!(f, "{}", qr_img)
    }
}

#[derive(Debug, PartialEq)]
pub struct QRPin {
    pub password: [u8; 4],
}

#[derive(Error, Debug)]

pub enum BadPinError {
    #[error("The PIN must consist of {PIN_LENGTH} digits, found {0}")]
    InvalidLength(usize),
    #[error("Invalid digit {0}")]
    InvalidDigit(char),
}

impl FromStr for QRPin {
    type Err = BadPinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().count() != PIN_LENGTH {
            return Err(BadPinError::InvalidLength(s.len()));
        }

        let mut pwd = [0u8; 4];
        for (i, digit) in s.chars().enumerate() {
            pwd[i] = digit.to_digit(10).ok_or(BadPinError::InvalidDigit(digit))? as u8;
        }
        Ok(QRPin { password: pwd })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pin_successfully() {
        for (pin, pwd) in &[
            ("0000", [0, 0, 0, 0]),
            ("1123", [1, 1, 2, 3]),
            ("0002", [0, 0, 0, 2]),
        ] {
            let qr_pin = QRPin::from_str(pin).unwrap();
            assert_eq!(qr_pin, QRPin { password: *pwd })
        }
    }
    #[test]
    fn pins_that_do_not_satisfy_length_reqs_return_error() {
        for bad_pin in &["", "1", "11", "111", "11111"] {
            let qr_pin = QRPin::from_str(bad_pin);
            assert!(qr_pin.is_err(),)
        }
    }

    #[test]
    fn pins_that_do_not_satisfy_content_reqs_return_error() {
        for bad_pin in &["    ", " 111", "llll", "000u"] {
            let qr_pin = QRPin::from_str(bad_pin);
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
