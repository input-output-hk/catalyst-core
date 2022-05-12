mod img;
mod payload;

use crate::cli::kedqr::QrCodeOpts;
use catalyst_toolbox::kedqr::QrPin;
pub use img::generate_qr;
pub use payload::generate_payload;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

/// QCode CLI toolkit
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct EncodeQrCodeCmd {
    /// Path to file containing ed25519extended bech32 value.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    /// Path to file to save qr code output, if not provided console output will be attempted.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// Pin code. 4-digit number is used on Catalyst.
    #[structopt(short, long, parse(try_from_str))]
    pin: QrPin,

    #[structopt(flatten)]
    opts: QrCodeOpts,
}

impl EncodeQrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        match self.opts {
            QrCodeOpts::Payload => generate_payload(self.input, self.output, self.pin),
            QrCodeOpts::Img => generate_qr(self.input, self.output, self.pin),
        }
    }
}
