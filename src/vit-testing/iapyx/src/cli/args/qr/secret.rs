use crate::PinReadMode;
use crate::QrReader;

use crate::cli::args::qr::IapyxQrCommandError;
use bech32::ToBase32;
use chain_crypto::AsymmetricKey;
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;
use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct GetSecretFromQrCommand {
    #[structopt(long = "qr")]
    pub qr: PathBuf,

    #[structopt(short = "p", long = "pin", default_value = "1234")]
    pub pin: String,

    #[structopt(short = "f", long = "read-from-filename")]
    pub read_pin_from_filename: bool,
}

impl GetSecretFromQrCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        println!("Decoding qr from file: {:?}...", self.qr);
        let pin_read_mode = PinReadMode::new(self.read_pin_from_filename, &self.pin);
        let pin_reader = QrReader::new(pin_read_mode);
        let secret = pin_reader.read_qr(&self.qr)?;
        let bin: [u8; 64] = secret.leak_secret().as_ref().try_into().unwrap();
        let secret_key: SecretKey<Ed25519Extended> = SecretKey::from_binary(&bin).unwrap();
        let hrp = Ed25519Extended::SECRET_BECH32_HRP;
        println!(
            "{}",
            bech32::encode(hrp, secret_key.leak_secret().to_base32())?
        );
        Ok(())
    }
}
