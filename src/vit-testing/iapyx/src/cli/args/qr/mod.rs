use crate::PinReadMode;
use crate::QrReader;
use crate::Wallet;
use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum IapyxQrCommandError {
    #[error("proxy error")]
    ProxyError(#[from] crate::backend::ProxyServerError),
}

#[derive(StructOpt, Debug)]
pub struct IapyxQrCommand {
    #[structopt(short = "q", long = "qr-codes-folder")]
    pub qr_codes_folder: PathBuf,

    #[structopt(short = "g", long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[structopt(short = "f", long = "read-from-filename")]
    pub read_pin_from_filename: bool,

    #[structopt(short = "s", long = "stop-at-fail")]
    pub stop_at_fail: bool,
}

impl IapyxQrCommand {
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
                Wallet::recover_from_utxo(secret.leak_secret().as_ref().try_into().unwrap())
                    .unwrap()
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
