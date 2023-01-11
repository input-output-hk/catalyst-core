mod img;
mod payload;

use crate::cli::kedqr::QrCodeOpts;
use catalyst_toolbox::kedqr::QrPin;
use color_eyre::Report;
pub use img::{save_secret_from_qr, secret_from_qr};
pub use payload::{decode_payload, secret_from_payload};
use std::{path::PathBuf, str::FromStr};
use clap::Parser;

/// QCode CLI toolkit
#[derive(Debug, PartialEq, Eq, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct DecodeQrCodeCmd {
    /// Path to file containing img or payload.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    input: PathBuf,
    /// Path to file to save secret output, if not provided console output will be attempted.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    output: Option<PathBuf>,
    /// Pin code. 4-digit number is used on Catalyst.
    #[clap(short, long, value_parser = QrPin::from_str)]
    pin: QrPin,

    #[clap(short, long, value_parser = QrCodeOpts::from_str)]
    opts: QrCodeOpts,
}

impl DecodeQrCodeCmd {
    pub fn exec(self) -> Result<(), Report> {
        match self.opts {
            QrCodeOpts::Payload => decode_payload(self.input, self.output, self.pin),
            QrCodeOpts::Img => save_secret_from_qr(self.input, self.output, self.pin),
        }
    }
}

