use std::io::Write;
use std::path::Path;
use std::process::Command;
use snapshot_trigger_service::config::NetworkType;

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

#[allow(dead_code)]
pub fn write_content<P: AsRef<Path>>(content: &str, path: P) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}