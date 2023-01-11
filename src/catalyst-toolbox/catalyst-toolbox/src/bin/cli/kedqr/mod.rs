mod decode;
mod encode;
mod info;
mod verify;

use std::str::FromStr;

use color_eyre::{Report, eyre::bail};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
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
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Self::Encode(encode) => encode.exec()?,
            Self::Decode(decode) => decode.exec()?,
            Self::Info(info) => info.exec()?,
            Self::Verify(verify) => verify.exec()?,
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[clap(rename_all = "kebab-case")]
pub enum QrCodeOpts {
    Img,
    Payload,
}

impl FromStr for QrCodeOpts {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "img" => Ok(Self::Img),
            "payload" => Ok(Self::Payload),
            other => bail!("unknown QrCodeOpts: {other}"),
        }
    }
}
