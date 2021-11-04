use bech32::ToBase32;
use catalyst_toolbox::kedqr::decode;
use catalyst_toolbox::kedqr::QrPin;
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use std::fs::File;
use std::io::BufRead;
use std::io::Write;
use std::{error::Error, fs::OpenOptions, io::BufReader, path::PathBuf};

pub fn decode_hash(
    input: PathBuf,
    output: Option<PathBuf>,
    pin: QrPin,
) -> Result<(), Box<dyn Error>> {
    // open input key and parse it
    let input = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .append(false)
        .open(&input)
        .expect("Could not open input file.");

    let mut reader = BufReader::new(input);
    let mut hash_str = String::new();
    let _len = reader
        .read_line(&mut hash_str)
        .expect("Could not read input file.");
    hash_str = hash_str.trim_end().to_string();

    // use parsed pin from args
    let pwd = pin.password;
    // generate qrcode with key and parsed pin
    let secret = decode(hash_str, &pwd)?;
    let hrp = Ed25519Extended::SECRET_BECH32_HRP;
    let secret_key = bech32::encode(hrp, secret.leak_secret().to_base32())?;
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
