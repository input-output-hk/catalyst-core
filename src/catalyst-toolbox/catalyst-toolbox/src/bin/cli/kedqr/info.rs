use super::QrCodeOpts;
use crate::cli::kedqr::decode::{secret_from_payload, secret_from_qr};
use catalyst_toolbox::kedqr::QrPin;
use chain_addr::{AddressReadable, Discrimination, Kind};
use chain_core::{
    mempack::{ReadBuf, Readable},
    property::Deserialize,
};
use chain_crypto::{Ed25519Extended, SecretKey};
use chain_impl_mockchain::block::Block;
use jormungandr_lib::interfaces::{Block0Configuration, Initial};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use url::Url;
#[derive(StructOpt, Debug)]
pub struct InfoForQrCodeCmd {
    /// Path to file containing img or payload.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// Pin code. 4-digit number is used on Catalyst.
    #[structopt(short, long, parse(try_from_str))]
    pin: QrPin,

    /// Blockchain block0. Can be either url of local file path
    #[structopt(long = "block0")]
    pub block0: Option<String>,

    /// Set the discrimination type to testing (default is production).
    #[structopt(short, long)]
    pub testing: bool,

    #[structopt(flatten)]
    opts: QrCodeOpts,
}

impl InfoForQrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn std::error::Error>> {
        let secret_key: SecretKey<Ed25519Extended> = {
            match self.opts {
                QrCodeOpts::Payload => secret_from_payload(&self.input, self.pin)?,
                QrCodeOpts::Img => secret_from_qr(&self.input, self.pin)?,
            }
        };
        let kind = Kind::Account(secret_key.to_public());
        let address = chain_addr::Address(test_discrimination(self.testing), kind);

        if let Some(block0_path) = &self.block0 {
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
                    panic!("invalid block0: should be either path to filesystem or url ");
                }
            };
            let genesis = Block0Configuration::from_block(&block)?;

            for initial in genesis.initial.iter() {
                if let Initial::Fund(initial_utxos) = initial {
                    if let Some(entry) = initial_utxos
                        .iter()
                        .find(|x| x.address == address.clone().into())
                    {
                        println!(
                            "Address corresponding to input qr found in block0: '{}' with value: '{}'", 
                            AddressReadable::from_address(&test_prefix(self.testing),&address), entry.value);
                        return Ok(());
                    }
                }
            }
            eprintln!("Address corresponding to input qr not found in block0");
        } else {
            println!(
                "Address: {}",
                AddressReadable::from_address(&test_prefix(self.testing), &address)
            );
        }
        Ok(())
    }
}

pub fn test_discrimination(testing: bool) -> Discrimination {
    match testing {
        false => Discrimination::Production,
        true => Discrimination::Test,
    }
}

pub fn test_prefix(testing: bool) -> String {
    match test_discrimination(testing) {
        Discrimination::Production => "ca".to_string(),
        Discrimination::Test => "ta".to_string(),
    }
}
