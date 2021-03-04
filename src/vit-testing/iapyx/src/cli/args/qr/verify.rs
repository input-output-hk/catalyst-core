use crate::cli::args::qr::IapyxQrCommandError;
use crate::PinReadMode;
use crate::QrReader;
use crate::Wallet;
use chain_addr::AddressReadable;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519Extended;
use chain_crypto::SecretKey;
use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct VerifyQrCommand {
    #[structopt(short = "q", long = "qr-codes-folder")]
    pub qr_codes_folder: PathBuf,

    #[structopt(short = "g", long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[structopt(short = "f", long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    #[structopt(short = "s", long = "stop-at-fail")]
    pub stop_at_fail: bool,
}

impl VerifyQrCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        println!("Decoding...");

        let pin_read_mode = PinReadMode::new(self.read_pin_from_filename, &self.global_pin);

        let qr_codes: Vec<PathBuf> = std::fs::read_dir(&self.qr_codes_folder)
            .unwrap()
            .into_iter()
            .map(|x| x.unwrap().path())
            .collect();

        let pin_reader = QrReader::new(pin_read_mode);
        let secrets = pin_reader.read_qrs(&qr_codes, self.stop_at_fail);
        let wallets: Vec<Wallet> = secrets
            .into_iter()
            .map(|secret| {
                let bin: [u8; 64] = secret.leak_secret().as_ref().try_into().unwrap();

                let secret_key: SecretKey<Ed25519Extended> = SecretKey::from_binary(&bin).unwrap();
                let kind = Kind::Single(secret_key.to_public());
                let address = chain_addr::Address(Discrimination::Production, kind);
                println!(
                    "{}",
                    AddressReadable::from_address("ca", &address).to_string()
                );

                Wallet::recover_from_utxo(&bin).unwrap()
            })
            .collect();

        println!(
            "{} QR read. {} succesfull, {} failed",
            qr_codes.len(),
            wallets.len(),
            qr_codes.len() - wallets.len()
        );
        Ok(())
    }
}
