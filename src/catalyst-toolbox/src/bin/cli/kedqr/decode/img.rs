use bech32::ToBase32;
use catalyst_toolbox::kedqr::KeyQrCode;
use catalyst_toolbox::kedqr::QrPin;
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn secret_from_qr(
    qr: PathBuf,
    output: Option<PathBuf>,
    pin: QrPin,
) -> Result<(), Box<dyn Error>> {
    let img = image::open(qr)?;
    let secret = KeyQrCode::decode(img, &pin.password)?;
    let sk = secret.first().unwrap().clone();
    let hrp = Ed25519Extended::SECRET_BECH32_HRP;
    let secret_key = bech32::encode(hrp, sk.leak_secret().to_base32())?;

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
