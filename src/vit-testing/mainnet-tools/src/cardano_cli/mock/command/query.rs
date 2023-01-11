use crate::cardano_cli::mock::command::write_to_file_or_println;
use crate::cardano_cli::mock::fake;
use std::io::Error;
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Query {
    Utxo(UTxOCommand),
    Tip(TipCommand),
    ProtocolParameters(ProtocolParametersCommand),
}

impl Query {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Self::Utxo(utxo) => utxo.exec(),
            Self::Tip(tip) => tip.exec(),
            Self::ProtocolParameters(protocol) => protocol.exec(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct ProtocolParametersCommand {
    #[clap(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[clap(long = "mainnet")]
    pub mainnet: bool,

    #[clap(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl ProtocolParametersCommand {
    pub fn exec(self) -> Result<(), Error> {
        assert!(
            !(self.mainnet || self.testnet_magic.is_some()),
            "no network setting"
        );

        let protocol_parameters = fake::protocol_parameters();
        write_to_file_or_println(
            self.out_file,
            &serde_json::to_string(&protocol_parameters).unwrap(),
        )
    }
}

#[derive(Parser, Debug)]
pub struct TipCommand {
    #[clap(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[clap(long = "mainnet")]
    pub mainnet: bool,

    #[clap(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl TipCommand {
    pub fn exec(self) -> Result<(), Error> {
        let tip = fake::tip();
        write_to_file_or_println(self.out_file, &serde_json::to_string(&tip).unwrap())
    }
}
#[derive(Parser, Debug)]
pub struct UTxOCommand {
    #[clap(long = "address")]
    pub address: String,

    #[clap(long = "testnet-magic")]
    pub testnet_magic: Option<u32>,

    #[clap(long = "mainnet")]
    pub mainnet: bool,

    #[clap(long = "out-file")]
    pub out_file: Option<PathBuf>,
}

impl UTxOCommand {
    pub fn exec(self) -> Result<(), Error> {
        let utxos = fake::utxo();
        write_to_file_or_println(self.out_file, &utxos.to_string())
    }
}
