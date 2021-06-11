use super::gang::{GroupElement, Scalar};
use super::encryption::Ciphertext;
use cryptoxide::digest::Digest;
use cryptoxide::sha2::Sha512;

/// Proof of discrete logarithm equivalence
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Proof {
    a1: GroupElement,
    a2: GroupElement,
    z: Scalar,
}

pub(crate) const PROOF_SIZE: usize = 162; // Scalar is 32 bytes

impl Proof {
    pub fn to_bytes(&self) -> [u8; PROOF_SIZE] {
        let mut output = [0u8; PROOF_SIZE];
        output[0..65].copy_from_slice(&self.a1.to_bytes());
        output[65..130].copy_from_slice(&self.a2.to_bytes());
        output[130..162].copy_from_slice(&self.z.to_bytes());
        output
    }

    pub fn to_slice_mut(&self, output: &mut [u8]) {
        assert_eq!(output.len(), PROOF_SIZE);
        output[0..65].copy_from_slice(&self.a1.to_bytes());
        output[65..130].copy_from_slice(&self.a2.to_bytes());
        output[130..162].copy_from_slice(&self.z.to_bytes());
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != PROOF_SIZE {
            return None;
        }
        let a1 = GroupElement::from_bytes(&slice[0..65])?;
        let a2 = GroupElement::from_bytes(&slice[65..130])?;
        let z = Scalar::from_bytes(&slice[130..162])?;

        let proof = Proof { a1, a2, z };
        Some(proof)
    }
}

/// Parameters for DLEQ where g1^a = h1, h2^a = h2
pub struct DecrNIZK<'a> {
    pub c: &'a Ciphertext,
    pub g: &'a GroupElement,
    pub h1: &'a GroupElement,
    pub g2: &'a GroupElement,
    pub h2: &'a GroupElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Challenge(Scalar);

fn challenge(
    c: &GroupElement,
    d: &GroupElement,
    a1: &GroupElement,
    a2: &GroupElement,
) -> Challenge {
    let mut out = [0u8; 64];
    let mut ctx = Sha512::new();
    ctx.input(&c.to_bytes());
    ctx.input(&d.to_bytes());
    ctx.input(&a1.to_bytes());
    ctx.input(&a2.to_bytes());
    ctx.result(&mut out);
    Challenge(Scalar::from_bytes(&out[0..32]).unwrap())
}

/// Generate a decryption zero knowledge proof
pub fn generate(w: &Scalar, share: &GroupElement, sk: &Scalar) -> Proof {
    let a1 = GroupElement::generator() * w;
    let a2 = share * w;
    let d = share * sk;
    let e = challenge(share, &d, &a1, &a2);
    let z = sk * &e.0 + w;

    Proof { a1, a2, z }
}

/// Verify a decryption zero knowledge proof
pub fn verify(
    share: &GroupElement,
    decrypted_share: &GroupElement,
    pk: &GroupElement,
    proof: &Proof,
) -> bool {
    let e = challenge(share, decrypted_share, &proof.a1, &proof.a2);
    let gz = GroupElement::generator() * &proof.z;
    let he = pk * &e.0;
    let he_a1 = he + &proof.a1;
    let c1z = share * &proof.z;
    let de = decrypted_share * &e.0;
    let de_a2 = de + &proof.a2;
    gz == he_a1 && c1z == de_a2
}

#[cfg(test)]
mod tests {
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    use super::{generate, verify, GroupElement, Scalar};

    #[test]
    pub fn it_works() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let sk = Scalar::random(&mut r);
        let w = Scalar::random(&mut r);
        let share_r = Scalar::random(&mut r);

        let pk = GroupElement::generator() * &sk;
        let share = GroupElement::generator() * &share_r;
        let decrypted_share = &share * &sk;

        let proof = generate(&w, &share, &sk);
        let verified = verify(&share, &decrypted_share, &pk, &proof);
        assert_eq!(verified, true);
    }
}
