use crate::config::Config;
use catalyst_toolbox::kedqr::{generate, KeyQrCode};
use chain_crypto::SecretKey;
use chain_impl_mockchain::key::EitherEd25519SecretKey;
use hersir::config::WalletTemplate;
use image::ImageError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;
use thor::{Wallet, WalletAlias};
use tracing::{info, span, trace, Level};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn generate_qr_and_hashes<P: AsRef<Path>>(
    wallets: Vec<(&WalletAlias, Wallet)>,
    initials: &HashMap<WalletTemplate, String>,
    parameters: &Config,
    folder: P,
) -> Result<(), Error> {
    let span = span!(Level::TRACE, "qr code generation");
    let _enter = span.enter();

    let total = wallets.len();
    let folder = folder.as_ref();

    info!(
        "generating {} valid qr codes and payloads...",
        wallets.len()
    );

    for (idx, (alias, wallet)) in wallets.iter().enumerate() {
        let pin = initials
            .iter()
            .find_map(|(template, pin)| {
                if template.alias() == Some(alias.to_string()) {
                    Some(pin)
                } else {
                    None
                }
            })
            .unwrap();
        let png = folder.join(format!("{}_{}.png", alias, pin));
        trace!("[{}/{}] Qr dumped to {:?}", idx + 1, total, png);
        wallet.save_qr_code(png, &pin_to_bytes(pin));

        let hash = folder.join(format!("{}_{}.txt", alias, pin));
        trace!("[{}/{}] QR payload dumped to {:?}", idx + 1, total, hash);
        wallet.save_qr_code_hash(hash, &pin_to_bytes(pin));
    }

    let zero_funds_initial_counts = parameters.initials.block0.zero_funds_count();

    if zero_funds_initial_counts > 0 {
        let zero_funds_pin = parameters.initials.block0.zero_funds_pin().unwrap();

        info!("generating {} fake qr codes", zero_funds_initial_counts);
        for i in 1..zero_funds_initial_counts + 1 {
            let sk = SecretKey::generate(rand::thread_rng());
            let qr = KeyQrCode::generate(sk.clone(), &pin_to_bytes(&zero_funds_pin));
            let img = qr.to_img();
            let png = folder.join(&format!("zero_funds_{}_{}.png", i, zero_funds_pin));
            trace!(
                "[{}/{}] Qr code dumped to {:?}",
                i + 1,
                zero_funds_initial_counts + 1,
                png
            );
            img.save(png)?;

            let hash = folder.join(format!("zero_funds_{}.txt", i));
            trace!(
                "[{}/{}] Qr payload dumped to {:?}",
                i + 1,
                zero_funds_initial_counts + 1,
                hash
            );
            std::fs::write(hash, generate(sk, &pin_to_bytes(&zero_funds_pin)))?;
        }
    }

    info!("qr codes generation done");
    Ok(())
}

pub fn pin_to_bytes(pin: &str) -> Vec<u8> {
    pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect()
}

pub trait WalletExtension {
    fn save_qr_code<P: AsRef<Path>>(&self, path: P, password: &[u8]);
    fn save_qr_code_hash<P: AsRef<Path>>(&self, path: P, password: &[u8]);
}

impl WalletExtension for Wallet {
    fn save_qr_code<P: AsRef<Path>>(&self, path: P, password: &[u8]) {
        let qr = match self {
            Wallet::Account(account) => {
                let secret_key = match account.signing_key().as_ref() {
                    EitherEd25519SecretKey::Extended(secret_key) => secret_key,
                    EitherEd25519SecretKey::Normal(_) => panic!("unsupported secret key type"),
                };
                KeyQrCode::generate(secret_key.clone(), password)
            }
            Wallet::UTxO(utxo) => {
                KeyQrCode::generate(utxo.last_signing_key().clone().into_secret_key(), password)
            }
            Wallet::Delegation(delegation) => KeyQrCode::generate(
                delegation.last_signing_key().clone().into_secret_key(),
                password,
            ),
        };

        qr.to_img().save(path).unwrap();
    }

    fn save_qr_code_hash<P: AsRef<Path>>(&self, path: P, password: &[u8]) {
        let qr = match self {
            Wallet::Account(account) => {
                let secret_key = match account.signing_key().as_ref() {
                    EitherEd25519SecretKey::Extended(secret_key) => secret_key,
                    EitherEd25519SecretKey::Normal(_) => panic!("unsupported secret key type"),
                };
                generate(secret_key.clone(), password)
            }
            Wallet::UTxO(utxo) => {
                generate(utxo.last_signing_key().clone().into_secret_key(), password)
            }
            Wallet::Delegation(delegation) => generate(
                delegation.last_signing_key().clone().into_secret_key(),
                password,
            ),
        };

        let mut file = File::create(path).unwrap();
        writeln!(file, "{}", qr).unwrap();
    }
}
