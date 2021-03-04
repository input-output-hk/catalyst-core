use crate::PinReadMode;
use crate::QrReader;

use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::cli::args::qr::IapyxQrCommandError;
use chain_addr::AddressReadable;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;

#[derive(StructOpt, Debug)]
pub struct GetAddressFromQRCommand {
    #[structopt(long = "qr")]
    pub qr: PathBuf,

    #[structopt(short = "p", long = "pin", default_value = "1234")]
    pub pin: String,

    #[structopt(short = "f", long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    // if true then testing discrimination would be used
    #[structopt(long = "testing")]
    pub testing: bool,

    // if true then testing discrimination would be used
    #[structopt(long = "prefix", default_value = "ca")]
    pub prefix: String,
}

impl GetAddressFromQRCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        println!("Decoding qr from file: {:?} ...", self.qr);
        let pin_read_mode = PinReadMode::new(self.read_pin_from_filename, &self.pin);
        let pin_reader = QrReader::new(pin_read_mode);
        let secret = pin_reader.read_qr(&self.qr)?;
        let bin: [u8; 64] = secret.leak_secret().as_ref().try_into().unwrap();
        let secret_key: SecretKey<Ed25519Extended> = SecretKey::from_binary(&bin).unwrap();
        let kind = Kind::Single(secret_key.to_public());
        let address = chain_addr::Address(Discrimination::Production, kind);
        println!(
            "Address: {}",
            AddressReadable::from_address(&self.prefix, &address).to_string()
        );
        Ok(())
    }
}
