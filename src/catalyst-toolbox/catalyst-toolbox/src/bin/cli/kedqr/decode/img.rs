use bech32::{ToBase32, Variant};
use catalyst_toolbox::kedqr::KeyQrCode;
use catalyst_toolbox::kedqr::QrPin;
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;
use color_eyre::Report;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub fn save_secret_from_qr(qr: PathBuf, output: Option<PathBuf>, pin: QrPin) -> Result<(), Report> {
    let sk = secret_from_qr(&qr, pin)?;
    let hrp = Ed25519Extended::SECRET_BECH32_HRP;
    let secret_key = bech32::encode(hrp, sk.leak_secret().to_base32(), Variant::Bech32)?;

    match output {
        Some(path) => {
            // save secret to file, or print to stdout if it fails
            let mut file = File::create(path)?;
            file.write_all(secret_key.as_bytes())?;
        }
        None => {
            // prints secret to stdout when no path is specified
            println!("{}", secret_key);
        }
    };
    Ok(())
}

pub fn secret_from_qr(
    qr: impl AsRef<Path>,
    pin: QrPin,
) -> Result<SecretKey<Ed25519Extended>, Report> {
    let img = image::open(qr)?;
    let secret = KeyQrCode::decode(img, &pin.password)?;
    Ok(secret.first().unwrap().clone())
}
