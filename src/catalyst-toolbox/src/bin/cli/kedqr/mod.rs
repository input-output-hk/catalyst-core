mod decode;
mod encode;

use std::error::Error;
use structopt::StructOpt;
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum QrCodeCmd {
    /// Encode qr code
    Encode(encode::EncodeQrCodeCmd),
    /// Decode qr code
    Decode(decode::DecodeQrCodeCmd),
}

impl QrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Encode(encode) => encode.exec()?,
            Self::Decode(decode) => decode.exec()?,
        };
        Ok(())
    }
}
