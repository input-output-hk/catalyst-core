mod hash;
mod img;

use catalyst_toolbox::kedqr::QrPin;
pub use hash::generate_hash;
pub use img::generate_qr;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

/// QCode CLI toolkit
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct QrCodeCmd {
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

impl QrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        match self.opts {
            QrCodeOpts::Hash => generate_hash(self.input, self.output, self.pin),
            QrCodeOpts::Img => generate_qr(self.input, self.output, self.pin),
        }
    }
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum QrCodeOpts {
    Img,
    Hash,
}
