use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(rename_all = "kebab")]
pub struct Args {
    /// Path to the genesis block (the block0) of the blockchain
    #[clap(long, short, value_parser = PathBuf::from_str)]
    pub genesis_block: PathBuf,

    /// Set the secret node config (in YAML format).
    #[clap(long, short, value_parser = PathBuf::from_str)]
    pub secret: Option<PathBuf>,

    /// Specifies the address the node will listen.
    #[clap(short = 'a', long = "listen-address")]
    pub listen_address: Option<SocketAddr>,
}
