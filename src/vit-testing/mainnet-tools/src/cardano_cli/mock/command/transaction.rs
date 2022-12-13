use crate::cardano_cli::mock::command::write_to_file_or_println;
use crate::cardano_cli::mock::fake;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Transaction {
    Id(IdCommand),
    Sign(SignCommand),
    Submit(SubmitCommand),
    Build(BuildCommand),
}

impl Transaction {
    pub fn exec(self) -> Result<(), io::Error> {
        match self {
            Self::Id(_id) => {
                IdCommand::exec();
                Ok(())
            }
            Self::Sign(sign) => sign.exec(),
            Self::Submit(_submit) => {
                SubmitCommand::exec();
                Ok(())
            }
            Self::Build(build) => build.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct IdCommand {
    #[structopt(long = "tx-file")]
    pub tx_file: PathBuf,
}

impl IdCommand {
    pub fn exec() {
        println!("{}", fake::hash());
    }
}

#[derive(StructOpt, Debug)]
pub struct SignCommand {
    #[structopt(long = "tx-body-file")]
    pub tx_body_file: PathBuf,

    #[structopt(long = "signing-key-file")]
    pub signing_key_file: PathBuf,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[structopt(long = "mainnet")]
    pub mainnet: bool,

    #[structopt(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl SignCommand {
    pub fn exec(self) -> Result<(), io::Error> {
        write_to_file_or_println(self.out_file, &fake::sign())
    }
}

#[derive(StructOpt, Debug)]
pub struct SubmitCommand {
    #[structopt(long = "tx-file")]
    pub tx_body_file: PathBuf,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[structopt(long = "mainnet")]
    pub mainnet: bool,
}

impl SubmitCommand {
    pub fn exec() {
        println!("{}", fake::submit());
    }
}

#[derive(StructOpt, Debug)]
pub struct BuildCommand {
    #[structopt(long = "mainnet")]
    pub mainnet: bool,

    #[structopt(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[structopt(long = "tx-in")]
    pub tx_in: String,

    #[structopt(long = "change-address")]
    pub change_address: String,

    #[structopt(long = "certificate-file")]
    pub certificate_file: PathBuf,

    #[structopt(long = "protocol-params-file")]
    pub protocol_params_file: PathBuf,

    #[structopt(long = "out-file")]
    pub out_file: PathBuf,

    #[structopt(long = "witness-override")]
    pub witness_override: u32,
}

impl BuildCommand {
    pub fn exec(self) -> std::io::Result<()> {
        let mut file = File::create(self.out_file)?;
        file.write_all(fake::transaction().as_bytes())
    }
}
