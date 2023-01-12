use crate::cardano_cli::mock::command::write_to_file_or_println;
use crate::cardano_cli::mock::fake;
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub enum StakeAddress {
    Build(BuildCommand),
    RegisterCertificate(RegistrationCertificateCommand),
}

impl StakeAddress {
    pub fn exec(self) -> Result<(), std::io::Error> {
        match self {
            Self::Build(build) => build.exec(),
            Self::RegisterCertificate(register) => register.exec(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct BuildCommand {
    #[clap(long = "stake-verification-key")]
    pub stake_verification_key: Option<String>,

    #[clap(long = "stake-verification-key-file")]
    pub stake_verification_key_file: Option<PathBuf>,

    #[clap(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[clap(long = "mainnet")]
    pub mainnet: bool,

    #[clap(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl BuildCommand {
    pub fn exec(self) -> Result<(), std::io::Error> {
        assert!(!(self.stake_verification_key.is_none() && self.stake_verification_key_file.is_none()),
                "either --stake-verification-key or --stake-verification-key-file option need to be defined ");
        write_to_file_or_println(self.out_file, &fake::stake_address())
    }
}

#[derive(Parser, Debug)]
pub struct RegistrationCertificateCommand {
    #[clap(long = "stake-verification-key-file")]
    pub stake_verification_key_file: PathBuf,

    #[clap(long = "out-file")]
    pub out_file: PathBuf,
}

impl RegistrationCertificateCommand {
    pub fn exec(self) -> Result<(), std::io::Error> {
        let mut file = File::create(self.out_file)?;
        file.write_all(fake::stake_address().as_bytes())
    }
}
