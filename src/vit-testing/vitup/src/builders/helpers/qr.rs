use crate::builders::helpers::wallet::WalletExtension;
use crate::config::VitStartParameters;
use crate::Result;
use assert_fs::fixture::ChildPath;
use assert_fs::fixture::PathChild;
use catalyst_toolbox::kedqr::{generate, KeyQrCode};
use chain_crypto::SecretKey;
use hersir::builder::WalletTemplate;
use jortestkit::prelude::append;
use std::collections::HashMap;
use thor::Wallet;
use thor::WalletAlias;

pub fn generate_qr_and_hashes(
    wallets: Vec<(&WalletAlias, Wallet)>,
    initials: &HashMap<WalletTemplate, String>,
    parameters: &VitStartParameters,
    folder: &ChildPath,
) -> Result<()> {
    let total = wallets.len();

    for (idx, (alias, wallet)) in wallets.iter().enumerate() {
        let pin = initials
            .iter()
            .find_map(|(template, pin)| {
                if template.alias() == *alias {
                    Some(pin)
                } else {
                    None
                }
            })
            .unwrap();
        let png = folder.child(format!("{}_{}.png", alias, pin));
        println!("[{}/{}] Qr dumped to {:?}", idx + 1, total, png.path());
        wallet.save_qr_code(png.path(), &pin_to_bytes(pin));

        let hash = folder.child(format!("{}_{}.txt", alias, pin));
        println!(
            "[{}/{}] QR hash dumped to {:?}",
            idx + 1,
            total,
            hash.path()
        );
        wallet.save_qr_code_hash(hash.path(), &pin_to_bytes(pin));
    }

    let zero_funds_initial_counts = parameters.initials.zero_funds_count();

    if zero_funds_initial_counts > 0 {
        let zero_funds_pin = parameters.initials.zero_funds_pin().unwrap();

        for i in 1..zero_funds_initial_counts + 1 {
            let sk = SecretKey::generate(rand::thread_rng());
            let qr = KeyQrCode::generate(sk.clone(), &pin_to_bytes(&zero_funds_pin));
            let img = qr.to_img();
            let png = folder.child(&format!("zero_funds_{}_{}.png", i, zero_funds_pin));
            img.save(png.path())?;

            let hash = folder.child(format!("zero_funds_{}.txt", i));
            append(hash, generate(sk, &pin_to_bytes(&zero_funds_pin)))?;
        }
    }
    Ok(())
}

pub fn pin_to_bytes(pin: &str) -> Vec<u8> {
    pin.chars().map(|x| x.to_digit(10).unwrap() as u8).collect()
}
