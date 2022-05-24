use chain_crypto::{Ed25519Extended, SecretKey};
use color_eyre::Report;
use symmetric_cipher::{decrypt, encrypt};

pub fn generate(key: SecretKey<Ed25519Extended>, password: &[u8]) -> String {
    let secret = key.leak_secret();
    let rng = rand::thread_rng();
    // this won't fail because we already know it's an ed25519extended key,
    // so it is safe to unwrap
    let enc = encrypt(password, secret.as_ref(), rng).unwrap();
    // Using binary would make the QR codes more compact and probably less
    // prone to scanning errors.
    hex::encode(enc)
}

pub fn decode<S: Into<String>>(
    payload: S,
    password: &[u8],
) -> Result<SecretKey<Ed25519Extended>, Report> {
    let encrypted_bytes = hex::decode(payload.into())?;
    let key = decrypt(password, &encrypted_bytes)?;
    Ok(SecretKey::from_binary(&key)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        const PASSWORD: &[u8] = &[1, 2, 3, 4];
        let sk = SecretKey::generate(rand::thread_rng());
        let hash = generate(sk.clone(), PASSWORD);
        assert_eq!(
            sk.leak_secret().as_ref(),
            decode(hash, PASSWORD).unwrap().leak_secret().as_ref()
        );
    }
}
