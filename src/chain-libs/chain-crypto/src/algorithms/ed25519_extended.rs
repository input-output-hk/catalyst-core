use crate::key::{AsymmetricKey, AsymmetricPublicKey, SecretKeyError, SecretKeySizeStatic};
use crate::sign::SigningAlgorithm;

use super::ed25519 as ei;

use cryptoxide::ed25519;
use rand_core::{CryptoRng, RngCore};

/// ED25519 Signing Algorithm with extended secret key
pub struct Ed25519Extended;

const EXTENDED_KEY_SIZE: usize = 64;

#[derive(Clone)]
pub struct ExtendedPriv([u8; EXTENDED_KEY_SIZE]);

impl AsRef<[u8]> for ExtendedPriv {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<[u8; EXTENDED_KEY_SIZE]> for ExtendedPriv {
    fn from(b: [u8; EXTENDED_KEY_SIZE]) -> ExtendedPriv {
        ExtendedPriv(b)
    }
}

impl AsymmetricKey for Ed25519Extended {
    type Secret = ExtendedPriv;
    type PubAlg = ei::Ed25519;

    const SECRET_BECH32_HRP: &'static str = "ed25519e_sk";

    fn generate<T: RngCore + CryptoRng>(mut rng: T) -> Self::Secret {
        let mut bytes = [0u8; EXTENDED_KEY_SIZE];
        rng.fill_bytes(&mut bytes);

        bytes[0] &= 0b1111_1000;
        bytes[31] &= 0b0011_1111;
        bytes[31] |= 0b0100_0000;
        ExtendedPriv(bytes)
    }

    fn compute_public(key: &Self::Secret) -> <Self::PubAlg as AsymmetricPublicKey>::Public {
        let pk = ed25519::to_public(&key.0);
        ei::Pub(pk)
    }

    fn secret_from_binary(data: &[u8]) -> Result<Self::Secret, SecretKeyError> {
        if data.len() != EXTENDED_KEY_SIZE {
            return Err(SecretKeyError::SizeInvalid);
        }
        let mut buf = [0; EXTENDED_KEY_SIZE];
        buf.clone_from_slice(data);
        // TODO structure check
        Ok(ExtendedPriv(buf))
    }
}

impl SecretKeySizeStatic for Ed25519Extended {
    const SECRET_KEY_SIZE: usize = EXTENDED_KEY_SIZE;
}

impl SigningAlgorithm for Ed25519Extended {
    fn sign(key: &Self::Secret, msg: &[u8]) -> ei::Sig {
        ei::Sig(ed25519::signature_extended(msg, &key.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::key::KeyPair;
    use crate::sign::test::{keypair_signing_ko, keypair_signing_ok};

    #[quickcheck]
    fn sign_ok(input: (KeyPair<Ed25519Extended>, Vec<u8>)) -> bool {
        keypair_signing_ok(input)
    }

    #[quickcheck]
    fn sign_ko(input: (KeyPair<Ed25519Extended>, KeyPair<Ed25519Extended>, Vec<u8>)) -> bool {
        keypair_signing_ko(input)
    }
}
