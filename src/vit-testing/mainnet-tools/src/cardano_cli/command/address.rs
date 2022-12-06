use crate::cardano_cli::command::write_to_file_or_println;
use crate::cardano_cli::fake;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Address {
    Build(BuildCommand),
}

impl Address {
    pub fn exec(self) -> Result<(), io::Error> {
        match self {
            Self::Build(build) => build.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct BuildCommand {
    #[structopt(long = "payment-verification-key")]
    pub payment_verification_key: Option<String>,

    #[structopt(long = "payment-verification-key-file")]
    pub payment_verification_key_file: Option<PathBuf>,

    #[structopt(long = "stake-verification-key")]
    pub stake_verification_key: Option<String>,

    #[structopt(long = "stake-verification-key-file")]
    pub stake_verification_key_file: Option<PathBuf>,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[structopt(long = "mainnet")]
    pub mainnet: bool,

    #[structopt(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl BuildCommand {
    pub fn exec(self) -> Result<(), io::Error> {
        assert!(!(self.stake_verification_key.is_none() && self.stake_verification_key_file.is_none()), "either --stake-verification-key or --stake-verification-key-file option need to be defined ");
        assert!(!(self.payment_verification_key.is_none() && self.payment_verification_key_file.is_none()), "either --payment-verification-key or --payment-verification-key-file option need to be defined ");

        write_to_file_or_println(self.out_file, &fake::address())
    }
}
