use crate::commitment::CommitmentKey;
use crate::gang::{GroupElement, Scalar};
use crate::gargamel::{self, PublicKey, SecretKey};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::chacha20::ChaCha20;
use cryptoxide::digest::Digest;

#[derive(Clone)]
pub struct Encrypted {
    e1: gargamel::Ciphertext,
    e2: Box<[u8]>,
}

fn bc_key(ck: &GroupElement) -> ChaCha20 {
    let mut out = [0u8; 44];
    let mut h = Blake2b::new(44);
    h.input(&ck.to_bytes());
    h.result(&mut out);
    ChaCha20::new(&out[0..32], &out[32..44])
}

fn bc_process(ck: &GroupElement, m: &[u8]) -> Vec<u8> {
    let mut key = bc_key(ck);
    let mut dat = m.to_vec();
    key.process_mut(&mut dat);
    dat
}

pub fn encrypt(pk: &PublicKey, ck: &CommitmentKey, m: &[u8], r: &Scalar) -> Encrypted {
    let e1 = gargamel::encrypt_point(pk, &ck.h, r);
    let e2 = bc_process(&ck.h, m).into_boxed_slice();
    Encrypted { e1, e2 }
}

#[allow(dead_code)]
pub fn decrypt(sk: &SecretKey, e: &Encrypted) -> Vec<u8> {
    let ck = gargamel::decrypt_point(sk, &e.e1);
    bc_process(&ck, &e.e2)
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
        let ck = CommitmentKey::generate(&mut r);
        let k = SecretKey::generate(&mut r);
        let k = Keypair::from_secretkey(k);

        let r = Scalar::random(&mut r);

        let m = [1, 3, 4, 5, 6, 7];

        let encrypted = encrypt(&k.public_key, &ck, &m, &r);
        let result = decrypt(&k.secret_key, &encrypted);

        assert_eq!(&m[..], &result[..])
    }
}
