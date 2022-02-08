use catalyst_toolbox::kedqr::generate;
use catalyst_toolbox::kedqr::KeyQrCode;
use chain_impl_mockchain::key::EitherEd25519SecretKey;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thor::Wallet;

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
