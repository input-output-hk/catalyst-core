mod decode;
mod encode;
mod info;
mod verify;

use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum QrCodeCmd {
    /// Encode qr code
    Encode(encode::EncodeQrCodeCmd),
    /// Decode qr code
    Decode(decode::DecodeQrCodeCmd),
    /// Prints information for qr code
    Info(info::InfoForQrCodeCmd),
    /// Validates qr code
    Verify(verify::VerifyQrCodeCmd),
}

impl QrCodeCmd {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Encode(encode) => encode.exec()?,
            Self::Decode(decode) => decode.exec()?,
            Self::Info(info) => info.exec()?,
            Self::Verify(verify) => verify.exec()?,
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum QrCodeOpts {
    Img,
    Payload,
}
