use crate::config::NetworkType;
use catalyst_toolbox::kedqr;
use chain_crypto::{Ed25519Extended, SecretKey};
use std::io::Write;
use std::path::{Path, PathBuf};
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

pub fn write_content<P: AsRef<Path>>(content: &str, path: P) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub trait SecretKeyFromQrCode {
    fn secret_key_from_qr_code(&self) -> SecretKey<Ed25519Extended>;
}

impl SecretKeyFromQrCode for PathBuf {
    fn secret_key_from_qr_code(&self) -> SecretKey<Ed25519Extended> {
        let img = image::open(self).unwrap();
        //TODO: send pin to registration service or extract it from qr code filename
        let secrets = kedqr::KeyQrCode::decode(img, &[1, 2, 3, 4]).unwrap();
        let key_qr_code = secrets.get(0).unwrap().clone();
        key_qr_code
    }
}

pub trait PinProvider {
    fn pin(&self) -> String;
}

impl PinProvider for PathBuf {
    fn pin(&self) -> String {
        let chars: Vec<char> = self
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .rev()
            .take(4)
            .collect();
        chars.iter().rev().collect()
    }
}
