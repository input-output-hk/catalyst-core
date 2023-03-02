use clap::Args;
use std::net::SocketAddr;

const ADDRESS_DEFAULT: &str = "0.0.0.0:3030";

#[derive(Args, Clone)]
pub struct Settings {
    /// Server binding address
    #[clap(long, default_value = ADDRESS_DEFAULT)]
    pub address: SocketAddr,
}
