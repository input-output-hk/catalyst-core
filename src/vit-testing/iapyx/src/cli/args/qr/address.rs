use crate::PinReadMode;
use crate::QrReader;

use crate::cli::args::qr::IapyxQrCommandError;
use chain_addr::AddressReadable;
use chain_addr::{Discrimination, Kind};
use chain_core::mempack::ReadBuf;
use chain_core::mempack::Readable;
use chain_core::property::Deserialize;
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;
use chain_impl_mockchain::block::Block;
use jormungandr_lib::interfaces::{Block0Configuration, Initial};
use std::convert::TryInto;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;
#[derive(StructOpt, Debug)]
pub struct GetAddressFromQrCommand {
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

    #[structopt(long = "block0")]
    pub block0: Option<String>,
}

impl GetAddressFromQrCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        println!("Decoding qr from file: {:?}...", self.qr);
        let pin_read_mode = PinReadMode::new(self.read_pin_from_filename, &self.pin);
        let pin_reader = QrReader::new(pin_read_mode);
        let secret = pin_reader.read_qr(&self.qr)?;
        let bin: [u8; 64] = secret.leak_secret().as_ref().try_into().unwrap();
        let secret_key: SecretKey<Ed25519Extended> = SecretKey::from_binary(&bin).unwrap();
        let kind = Kind::Single(secret_key.to_public());
        let address = chain_addr::Address(Discrimination::Production, kind);

        if let Some(block0_path) = &self.block0 {
            println!("Reading block0 from location {:?}...", block0_path);

            let block = {
                if Path::new(block0_path).exists() {
                    let reader = std::fs::OpenOptions::new()
                        .create(false)
                        .write(false)
                        .read(true)
                        .append(false)
                        .open(block0_path)?;
                    let reader = BufReader::new(reader);
                    Block::deserialize(reader)?
                } else if Url::parse(block0_path).is_ok() {
                    let response = reqwest::blocking::get(block0_path)?;
                    let block0_bytes = response.bytes()?.to_vec();
                    Block::read(&mut ReadBuf::from(&block0_bytes))?
                } else {
                    panic!(" block0 should be either path to filesystem or url ");
                }
            };
            let genesis = Block0Configuration::from_block(&block)?;

            for initial in genesis.initial.iter() {
                if let Initial::Fund(initial_utxos) = initial {
                    if let Some(entry) = initial_utxos.iter().find(|x| {
                        let entry_address: chain_addr::Address = x.address.clone().into();
                        entry_address == address
                    }) {
                        println!(
                            "Address corresponding to input qr found in block0: '{}' with value: '{}'", 
                            AddressReadable::from_address(&self.prefix, &entry.address.clone().into()).to_string(),entry.value
                        );
                        return Ok(());
                    }
                }
            }
            println!("Address corresponding to input qr not found in block0");
        } else {
            println!(
                "Address: {}",
                AddressReadable::from_address(&self.prefix, &address).to_string()
            );
        }
        Ok(())
    }
}
