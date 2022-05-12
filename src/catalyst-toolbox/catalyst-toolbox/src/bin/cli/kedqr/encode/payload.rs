use catalyst_toolbox::kedqr::generate;
use catalyst_toolbox::kedqr::QrPin;
use chain_crypto::bech32::Bech32;
use chain_crypto::{Ed25519Extended, SecretKey};
use std::fs::File;
use std::io::Write;
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub fn generate_payload(
    input: PathBuf,
    output: Option<PathBuf>,
    pin: QrPin,
) -> Result<(), Box<dyn Error>> {
    // open input key and parse it
    let key_file = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .append(false)
        .open(&input)
        .expect("Could not open input file.");

    let mut reader = BufReader::new(key_file);
    let mut key_str = String::new();
    let _key_len = reader
        .read_line(&mut key_str)
        .expect("Could not read input file.");
    let sk = key_str.trim_end().to_string();

    let secret_key: SecretKey<Ed25519Extended> =
        SecretKey::try_from_bech32_str(&sk).expect("Malformed secret key.");
    // use parsed pin from args
    let pwd = pin.password;
    // generate qrcode with key and parsed pin
    let qr = generate(secret_key, &pwd);
    // process output
    match output {
        Some(path) => {
            // save qr code to file, or print to stdout if it fails
            let mut file = File::create(path)?;
            file.write_all(qr.as_bytes())?;
        }
        None => {
            // prints qr code to stdout when no path is specified
            println!("{}", qr);
        }
    }
    Ok(())
}
