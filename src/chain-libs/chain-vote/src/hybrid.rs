use crate::gang::{GroupElement, Scalar};
use crate::gargamel::{self, PublicKey, SecretKey};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::chacha20::ChaCha20;
use cryptoxide::digest::Digest;
use rand_core::{CryptoRng, RngCore};

#[derive(Clone)]
pub struct HybridCiphertext {
    // ElGamal Ciphertext
    e1: gargamel::Ciphertext,
    // Symmetric encrypted message
    e2: Box<[u8]>,
}

/// The hybrid encryption scheme uses a group element as a
/// representation of the symmetric key. This facilitates
/// its exchange using ElGamal encryption.
pub struct SymmetricKey {
    group_repr: GroupElement,
}

impl SymmetricKey {
    /// Generate a new random symmetric key
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let exponent = Scalar::random(rng);
        SymmetricKey {
            group_repr: GroupElement::generator() * &exponent,
        }
    }

    /// Initialise encryption, by hashing the group element
    fn initialise_encryption(&self) -> ChaCha20 {
        let mut out = [0u8; 44];
        let mut h = Blake2b::new(44);
        h.input(&self.group_repr.to_bytes());
        h.result(&mut out);
        ChaCha20::new(&out[0..32], &out[32..44])
    }

    /// Encrypt/decrypt a message using the symmetric key
    fn process(&self, m: &[u8]) -> Vec<u8> {
        let mut key = self.initialise_encryption();
        let mut dat = m.to_vec();
        key.process_mut(&mut dat);
        dat
    }
}

/// Encrypt a message using hybrid encryption
pub fn hybrid_encrypt(
    pk: &PublicKey,
    symmetric_key: &SymmetricKey,
    m: &[u8],
    r: &Scalar,
) -> HybridCiphertext {
    let e1 = gargamel::encrypt_point(pk, &symmetric_key.group_repr, r);
    let e2 = symmetric_key.process(m).into_boxed_slice();
    HybridCiphertext { e1, e2 }
}

#[allow(dead_code)]
/// Encrypt a message using hybrid decryption
pub fn hybrid_decrypt(sk: &SecretKey, ciphertext: &HybridCiphertext) -> Vec<u8> {
    let symmetric_key = SymmetricKey {
        group_repr: gargamel::decrypt_point(sk, &ciphertext.e1),
    };

    symmetric_key.process(&ciphertext.e2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gargamel::Keypair;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn encrypt_decrypt() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let k = SecretKey::generate(&mut r);
        let k = Keypair::from_secretkey(k);

        let sym_key = SymmetricKey::new(&mut r);

        let r = Scalar::random(&mut r);

        let m = [1, 3, 4, 5, 6, 7];

        let encrypted = hybrid_encrypt(&k.public_key, &sym_key, &m, &r);
        let result = hybrid_decrypt(&k.secret_key, &encrypted);

        assert_eq!(&m[..], &result[..])
    }
}
