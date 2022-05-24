use bech32::{ToBase32, Variant};
use catalyst_toolbox::kedqr::{decode, QrPin};
use chain_crypto::{AsymmetricKey, Ed25519Extended, SecretKey};
use color_eyre::Report;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

pub fn decode_payload(
    input: PathBuf,
    output: Option<PathBuf>,
    pin: QrPin,
) -> Result<(), Report> {
    // generate qrcode with key and parsed pin
    let secret = secret_from_payload(input, pin)?;
    let hrp = Ed25519Extended::SECRET_BECH32_HRP;
    let secret_key = bech32::encode(hrp, secret.leak_secret().to_base32(), Variant::Bech32)?;
    // process output
    match output {
        Some(path) => {
            // save qr code to file, or print to stdout if it fails
            let mut file = File::create(path)?;
            file.write_all(secret_key.as_bytes())?;
        }
        None => {
            // prints qr code to stdout when no path is specified
            println!("{}", secret_key);
        }
    }
    Ok(())
}

pub fn secret_from_payload(
    input: impl AsRef<Path>,
    pin: QrPin,
) -> Result<SecretKey<Ed25519Extended>, Report> {
    let input = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .append(false)
        .open(&input)
        .expect("Could not open input file.");

    let mut reader = BufReader::new(input);
    let mut payload_str = String::new();
    let _len = reader
        .read_line(&mut payload_str)
        .expect("Could not read input file.");
    payload_str = payload_str.trim_end().to_string();

    // use parsed pin from args
    let pwd = pin.password;
    // generate qrcode with key and parsed pin
    decode(payload_str, &pwd)
}
