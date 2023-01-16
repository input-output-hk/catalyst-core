mod img;
mod payload;

use crate::cli::kedqr::QrCodeOpts;
use catalyst_toolbox::kedqr::QrPin;
use clap::Parser;
use color_eyre::Report;
pub use img::generate_qr;
pub use payload::generate_payload;
use std::{path::PathBuf, str::FromStr};

/// QCode CLI toolkit
#[derive(Debug, PartialEq, Eq, Parser)]
#[clap(rename_all = "kebab-case")]
pub struct EncodeQrCodeCmd {
    /// Path to file containing ed25519extended bech32 value.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    input: PathBuf,
    /// Path to file to save qr code output, if not provided console output will be attempted.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    output: Option<PathBuf>,
    /// Pin code. 4-digit number is used on Catalyst.
    #[clap(short, long, value_parser = QrPin::from_str)]
    pin: QrPin,

    #[clap(short, long, value_parser = QrCodeOpts::from_str)]
    opts: QrCodeOpts,
}

impl EncodeQrCodeCmd {
    pub fn exec(self) -> Result<(), Report> {
        match self.opts {
            QrCodeOpts::Payload => generate_payload(self.input, self.output, self.pin),
            QrCodeOpts::Img => generate_qr(self.input, self.output, self.pin),
        }
    }
}
