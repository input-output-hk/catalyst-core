use catalyst_toolbox::kedqr::{KeyQrCode, QRPin};
use chain_crypto::bech32::Bech32;
use chain_crypto::{Ed25519Extended, SecretKey};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use structopt::StructOpt;

/// QCode CLI toolkit
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct QRcodeApp {
    /// Path to file containing ed25519extended bech32 value.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    /// Path to file to save qr code output, if not provided console output will be attempted.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// Pin code. 4-digit number is used on Catalyst.
    #[structopt(short, long, parse(try_from_str))]
    pin: QRPin,
}

impl QRcodeApp {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        let QRcodeApp { input, output, pin } = self;
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
        let qr = KeyQrCode::generate(secret_key, &pwd);
        // process output
        match output {
            Some(path) => {
                // save qr code to file, or print to stdout if it fails
                let img = qr.to_img();
                if let Err(e) = img.save(path) {
                    println!("Error: {}", e);
                    println!();
                    println!("{}", qr);
                }
            }
            None => {
                // prints qr code to stdout when no path is specified
                println!();
                println!("{}", qr);
            }
        }
        Ok(())
    }
}
