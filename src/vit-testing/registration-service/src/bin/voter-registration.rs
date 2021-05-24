use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use futures::future::FutureExt;
use structopt::StructOpt;
use thiserror::Error;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli_future = tokio::task::spawn_blocking(|| VoterRegistrationCommand::from_args().exec())
        .map(|res| res.expect("CLI command failed for an unknown reason"))
        .fuse();

    signals_handler::with_signal_handler(cli_future).await
}

#[derive(StructOpt, Debug)]
pub struct VoterRegistrationCommand {
    #[structopt(long = "payment-signing-key")]
    pub payment_signing_key: PathBuf,

    #[structopt(long = "stake-signing-key")]
    pub stake_signing_key: PathBuf,

    #[structopt(long = "vote-public-key")]
    pub vote_public_key: PathBuf,

    #[structopt(long = "payment-address")]
    pub payment_address: String,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[structopt(long = "mainnet")]
    pub mainnet: bool,

    #[structopt(long = "mary-era")]
    pub mary_era: bool,

    #[structopt(long = "cardano-mode")]
    pub cardano_mode: bool,

    #[structopt(long = "sign")]
    pub sign: bool,

    #[structopt(long = "out-file")]
    pub out_file: PathBuf,
}

impl VoterRegistrationCommand {
    pub fn exec(self) -> Result<(), Error> {
        println!("Executed with parameters: {:#?}", self);

        if !self.payment_signing_key.exists() {
            return Err(Error::PaymentSigningKey);
        }
        if !self.stake_signing_key.exists() {
            return Err(Error::StakeSigningKey);
        }
        if !self.vote_public_key.exists() {
            return Err(Error::VotePublicKey);
        }
        write_to_output(&self.out_file)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("payment-signing-key: file does not exists")]
    PaymentSigningKey,
    #[error("stake-signing-key: file does not exists")]
    StakeSigningKey,
    #[error("vote-public-key: file does not exists")]
    VotePublicKey,
    #[error("cannot create output file")]
    IoError(#[from] std::io::Error),
}

fn write_to_output<P: AsRef<Path>>(file_path: P) -> Result<(), Error> {
    let content =  "{\
        \"type\": \"TxSignedShelley\", \
        \"description\": \",\
        \"cborHex\": \"83a500828258205761bdc4fd016ee0d52ac759ae6c0e8e0943d4892474283866a07f9768e48fee00825820e6701be50c87d8d584985edd4cf39799e1445bd37907027c44d08c7da79ea23200018182583900fec5a902e307707b6ab3de38104918c0e33cf4c3408e6fcea4f0a199c13582aec9a44fcc6d984be003c5058c660e1d2ff1370fd8b49ba73f1b00001e0369444cd7021a0002c329031a00ce0fc70758202386abf617780a925495f38f23d7bc594920ff374f03f3d7517a4345e355b047a1008182582099d1d0c4cdc8a4b206066e9606c6c3729678bd7338a8eab9bffdffa39d3df9585840af346c11fe7a222008f5b1b50fbc23a0cbc3d783bf4461f21353e8b5eb664adadb34291197e039e467d2a68346921879d1212bd0d54245a9e110162ecae9190ba219ef64a201582071ce673ef64b4ac1fb758b65df01b036665d4498256335e93e28b869568d9ed80258209be513df12b3fabe7c1b8c3f9fab0968eb2168d5689bf981c2f7c35b11718b2719ef65a101584057267d94e5bae64fa236924b83ce7411fef10bd5d73aca7af8403053cf2dc2e3621f7d253bf90933e2bc0bfb56146cf0a13925d9f96d6d06b0b798bc41d4000d\"\
    }";

    let mut file = File::create(file_path.as_ref())?;
    Ok(file.write_all(content.as_bytes())?)
}
