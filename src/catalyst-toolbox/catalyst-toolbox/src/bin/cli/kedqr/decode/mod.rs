mod hash;
mod img;

use crate::cli::kedqr::encode::QrCodeOpts;
use catalyst_toolbox::kedqr::QrPin;
pub use hash::decode_hash;
pub use img::secret_from_qr;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

/// QCode CLI toolkit
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct DecodeQrCodeCmd {
    /// Path to file containing img or hash.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    /// Path to file to save secret output, if not provided console output will be attempted.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// Pin code. 4-digit number is used on Catalyst.
    #[structopt(short, long, parse(try_from_str))]
    pin: QrPin,

    #[structopt(flatten)]
    opts: QrCodeOpts,
}

impl DecodeQrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        match self.opts {
            QrCodeOpts::Hash => decode_hash(self.input, self.output, self.pin),
            QrCodeOpts::Img => secret_from_qr(self.input, self.output, self.pin),
        }
    }
}
