use crate::config::NetworkType;
use std::process::Command;

pub trait CommandExt {
    fn arg_network(&mut self, network: NetworkType) -> &mut Self;
}

impl CommandExt for Command {
    fn arg_network(&mut self, network: NetworkType) -> &mut Self {
        match network {
            NetworkType::Mainnet => self.arg("--mainnet"),
            NetworkType::Testnet(magic) => self.arg("--testnet-magic").arg(magic.to_string()),
        }
    }
}
