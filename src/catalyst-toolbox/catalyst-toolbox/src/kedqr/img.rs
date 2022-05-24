use super::payload;
use chain_crypto::{Ed25519Extended, SecretKey};
use color_eyre::Report;
use image::{DynamicImage, ImageBuffer, Luma};
use qrcode::{
    render::{svg, unicode},
    EcLevel, QrCode,
};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct KeyQrCode {
    inner: QrCode,
}

impl KeyQrCode {
    pub fn generate(key: SecretKey<Ed25519Extended>, password: &[u8]) -> Self {
        let enc_hex = payload::generate(key, password);
        let inner = QrCode::with_error_correction_level(&enc_hex, EcLevel::H).unwrap();

        KeyQrCode { inner }
    }

    pub fn write_svg(&self, path: impl AsRef<Path>) -> Result<(), Report> {
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
    ) -> Result<Vec<SecretKey<Ed25519Extended>>, Report> {
        let mut decoder = quircs::Quirc::default();

        let img = img.into_luma8();

        let codes = decoder.identify(img.width() as usize, img.height() as usize, &img);

        codes
            .map(|code| -> Result<_, Report> {
                let decoded = code?.decode()?;

                // TODO: I actually don't know if this can fail
                let h = std::str::from_utf8(&decoded.payload)?;
                payload::decode(h, password)
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Improve into an integration test using a temporary directory.
    // Leaving here as an example.
    #[test]
    #[ignore]
    fn generate_svg() {
        const PASSWORD: &[u8] = &[1, 2, 3, 4];
        let sk = SecretKey::generate(rand::thread_rng());
        let qr = KeyQrCode::generate(sk, PASSWORD);
        qr.write_svg("qr-code.svg").unwrap();
    }

    #[test]
    #[ignore]
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
