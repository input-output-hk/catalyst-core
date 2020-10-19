use chain_ser::mempack::{ReadBuf, ReadError};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use rand_core::{CryptoRng, RngCore};
use typed_bytes::ByteBuilder;

use crate::commitment::{Commitment, CommitmentKey, COMMITMENT_BYTES_LEN};
use crate::encrypted::{EncryptingVote, PTP};
use crate::gang::{GroupElement, Scalar};
use crate::gargamel::{encrypt, Ciphertext, PublicKey, CIPHERTEXT_BYTES_LEN};
use crate::math::Polynomial;
use crate::unit_vector::binrep;

struct ABCD {
    alpha: Scalar,
    beta: Scalar,
    gamma: Scalar,
    delta: Scalar,
}

impl ABCD {
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let alpha = Scalar::random(rng);
        let beta = Scalar::random(rng);
        let gamma = Scalar::random(rng);
        let delta = Scalar::random(rng);
        ABCD {
            alpha,
            beta,
            gamma,
            delta,
        }
    }
}

/// I, B, A commitments
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct IBA {
    pub i: Commitment,
    pub b: Commitment,
    pub a: Commitment,
}

// Computed z, w, v
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ZWV {
    pub z: Scalar,
    pub w: Scalar,
    pub v: Scalar,
}

/// Proof of unit vector
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Proof {
    ibas: Vec<IBA>,
    ds: Vec<Ciphertext>,
    zwvs: Vec<ZWV>,
    r: Scalar,
}

impl IBA {
    pub(crate) fn serialize_in<T>(&self, bb: ByteBuilder<T>) -> ByteBuilder<T> {
        let mut bb = bb;
        for component in [&self.i, &self.b, &self.a].iter() {
            let buf = component.to_bytes();
            bb = bb.bytes(&buf);
        }
        bb
    }

    pub(crate) fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let i_buf = buf.get_slice(COMMITMENT_BYTES_LEN)?;
        let b_buf = buf.get_slice(COMMITMENT_BYTES_LEN)?;
        let a_buf = buf.get_slice(COMMITMENT_BYTES_LEN)?;
        Ok(Self {
            i: Commitment::from_bytes(i_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid IBA component".to_string()))?,
            b: Commitment::from_bytes(b_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid IBA component".to_string()))?,
            a: Commitment::from_bytes(a_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid IBA component".to_string()))?,
        })
    }
}

impl Proof {
    pub fn serialize_in<T>(&self, bb: ByteBuilder<T>) -> ByteBuilder<T> {
        let mut bb = bb;
        // serialize ibas
        bb = bb.u64(self.ibas.len() as u64);
        for iba in self.ibas.iter() {
            bb = iba.serialize_in(bb);
        }
        // serialize ciphertexts
        bb = bb.u64(self.ds.len() as u64);
        for ct in self.ds.iter() {
            let buf = ct.to_bytes();
            bb = bb.bytes(&buf);
        }
        // serialize zwvs
        bb = bb.u64(self.zwvs.len() as u64);
        for zwv in self.zwvs.iter() {
            bb = bb.bytes(&zwv.z.to_bytes());
            bb = bb.bytes(&zwv.w.to_bytes());
            bb = bb.bytes(&zwv.v.to_bytes());
        }
        // serialize r scalar
        bb = bb.bytes(&self.r.to_bytes());
        bb
    }

    pub fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let ibas_size = buf.get_u64()?;
        let mut ibas: Vec<IBA> = Vec::new();
        for _ in 0..ibas_size {
            ibas.push(IBA::read(buf)?);
        }

        let cts_size = buf.get_u64()?;
        let mut ds: Vec<Ciphertext> = Vec::new();
        for _ in 0..cts_size {
            let ct_buf = buf.get_slice(CIPHERTEXT_BYTES_LEN)?;
            ds.push(Ciphertext::from_bytes(ct_buf).ok_or_else(|| {
                ReadError::StructureInvalid("Invalid encoded ciphertext".to_string())
            })?);
        }

        let zwvs_size = buf.get_u64()?;
        let mut zwvs: Vec<ZWV> = Vec::new();
        for _ in 0..zwvs_size {
            zwvs.push(ZWV {
                z: Scalar::from_slice(buf.get_slice(32)?).ok_or_else(|| {
                    ReadError::StructureInvalid("Invalid ZWV encoded scalar Z".to_string())
                })?,
                w: Scalar::from_slice(buf.get_slice(32)?).ok_or_else(|| {
                    ReadError::StructureInvalid("Invalid ZWV encoded scalar W".to_string())
                })?,
                v: Scalar::from_slice(buf.get_slice(32)?).ok_or_else(|| {
                    ReadError::StructureInvalid("Invalid ZWV encoded scalar V".to_string())
                })?,
            });
        }

        let r = Scalar::from_slice(buf.get_slice(32)?).ok_or_else(|| {
            ReadError::StructureInvalid("Invalid Proof encoded R scalar".to_string())
        })?;
        Ok(Self { ibas, ds, zwvs, r })
    }
}

fn commitkey(pk: &PublicKey) -> CommitmentKey {
    let mut ctx = Blake2b::new(32);
    ctx.input(&pk.to_bytes());
    let mut i = 1u32;
    let mut h = [0u8; 32];
    loop {
        ctx.input(&i.to_be_bytes());
        ctx.result(&mut h);
        match Scalar::from_bytes(&h) {
            None => i += 1,
            Some(fe) => {
                let h = &GroupElement::generator() * &fe;
                break CommitmentKey { h };
            }
        }
    }
}

impl IBA {
    pub fn new(ck: &CommitmentKey, abcd: &ABCD, index: &Scalar) -> Self {
        assert!(index == &Scalar::zero() || index == &Scalar::one());

        // commit index bit: 0 or 1
        let i = Commitment::new(&ck, &index, &abcd.alpha);
        // commit beta
        let b = Commitment::new(&ck, &abcd.beta, &abcd.gamma);
        // commit i * B => 0 * B = 0 or 1 * B = B
        let acommited = if index == &Scalar::one() {
            abcd.beta.clone()
        } else {
            Scalar::zero()
        };
        let a = Commitment::new(&ck, &acommited, &abcd.delta);

        IBA { i, b, a }
    }
}

struct ChallengeContext(Blake2b);

fn hash_to_scalar(b: &Blake2b) -> Scalar {
    let mut h = [0u8; 32];
    b.clone().result(&mut h);
    Scalar::from_bytes(&h).unwrap()
}

impl ChallengeContext {
    fn new(public_key: &PublicKey, ciphers: &[Ciphertext], ibas: &[IBA]) -> Self {
        let mut ctx = Blake2b::new(32);
        ctx.input(&public_key.to_bytes());
        for c in ciphers {
            ctx.input(&c.to_bytes());
        }
        for iba in ibas {
            ctx.input(&iba.i.to_bytes());
            ctx.input(&iba.b.to_bytes());
            ctx.input(&iba.a.to_bytes());
        }
        ChallengeContext(ctx)
    }

    fn first_challenge(&self) -> Scalar {
        hash_to_scalar(&self.0)
    }

    fn second_challenge(&self, ds: &[Ciphertext]) -> Scalar {
        let mut x = self.0.clone();
        for d in ds {
            x.input(&d.to_bytes())
        }
        hash_to_scalar(&x)
    }
}

pub fn prove<R: RngCore + CryptoRng>(
    rng: &mut R,
    public_key: &PublicKey,
    encrypting_vote: EncryptingVote,
) -> Proof {
    let ciphers = PTP::new(encrypting_vote.ciphertexts, Ciphertext::zero);
    let cipher_randoms = PTP::new(encrypting_vote.random_elements, Scalar::zero);

    assert_eq!(ciphers.bits(), cipher_randoms.bits());

    let bits = ciphers.bits();

    let ck = commitkey(&public_key);

    let mut abcds = Vec::with_capacity(bits);
    for _ in 0..bits {
        abcds.push(ABCD::random(rng))
    }
    assert_eq!(abcds.len(), bits);

    let unit_vector = &encrypting_vote.unit_vector;
    let idx = binrep(unit_vector.ith(), bits as u32);
    assert_eq!(idx.len(), bits);

    // Generate I, B, A commitments
    let ibas: Vec<IBA> = abcds
        .iter()
        .zip(idx.iter())
        .map(|(abcd, index)| IBA::new(&ck, abcd, &(*index).into()))
        .collect();
    assert_eq!(ibas.len(), bits);

    // Generate First verifier challenge
    let cc = ChallengeContext::new(public_key, ciphers.as_ref(), &ibas);
    let cy = cc.first_challenge();

    let (ds, rs) = {
        // Compute polynomials pj(x)
        let polys = idx
            .iter()
            .zip(abcds.iter())
            .map(|(ix, abcd)| {
                let z1 = Polynomial::new(bits).set2(abcd.beta.clone(), (*ix).into());
                let z0 = Polynomial::new(bits).set2(abcd.beta.negate(), (!ix).into());
                (z0, z1)
            })
            .collect::<Vec<_>>();

        let mut pjs = Vec::new();
        for i in 0..ciphers.len() {
            let j = binrep(i, bits as u32);

            let mut acc = if j[0] {
                polys[0].1.clone()
            } else {
                polys[0].0.clone()
            };
            for k in 1..bits {
                let t = if j[k] {
                    polys[k].1.clone()
                } else {
                    polys[k].0.clone()
                };
                acc = acc * t;
            }
            pjs.push(acc)
        }

        assert_eq!(pjs.len(), ciphers.len());

        // Generate new Rs for Ds
        let mut rs = Vec::with_capacity(bits);
        for _ in 0..bits {
            let r = Scalar::random(rng);
            rs.push(r);
        }

        // Compute Ds
        let ds = rs
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let mut sum = Scalar::zero();
                #[allow(clippy::needless_range_loop)]
                for j in 0..ciphers.len() {
                    sum = sum + (cy.power(j) * pjs[j].get_coefficient_at(i))
                }

                encrypt(public_key, &sum, r)
            })
            .collect::<Vec<_>>();

        (ds, rs)
    };

    // Generate second verifier challenge
    let cx = cc.second_challenge(&ds);

    // Compute ZWVs
    let zwvs = abcds
        .iter()
        .zip(idx.iter())
        .map(|(abcd, index)| {
            let z = Scalar::from(*index) * &cx + &abcd.beta;
            let w = &abcd.alpha * &cx + &abcd.gamma;
            let v = &abcd.alpha * (&cx - &z) + &abcd.delta;
            ZWV { z, w, v }
        })
        .collect();

    // Compute R
    let r = {
        let cx_pow = cx.power(bits);
        let p1 = cipher_randoms
            .iter()
            .enumerate()
            .fold(Scalar::zero(), |acc, (i, r)| {
                let el = r * &cx_pow * cy.power(i);
                el + acc
            });
        let p2 = rs.iter().enumerate().fold(Scalar::zero(), |acc, (l, r)| {
            let el = r * cx.power(l);
            el + acc
        });
        p1 + p2
    };

    Proof { ibas, ds, zwvs, r }
}

pub fn verify(public_key: &PublicKey, ciphertexts: &[Ciphertext], proof: &Proof) -> bool {
    let ck = commitkey(&public_key);

    let ciphertexts = PTP::new(ciphertexts.to_vec(), Ciphertext::zero);
    let bits = ciphertexts.bits();
    let cc = ChallengeContext::new(public_key, ciphertexts.as_ref(), &proof.ibas);
    let cy = cc.first_challenge();
    let cx = cc.second_challenge(&proof.ds);

    if proof.ibas.len() != bits {
        return false;
    }

    if proof.zwvs.len() != bits {
        return false;
    }

    // check commitments are 0 / 1
    for (iba, zwv) in proof.ibas.iter().zip(proof.zwvs.iter()) {
        let com1 = Commitment::new(&ck, &zwv.z, &zwv.w);
        let lhs = &iba.i * &cx + &iba.b;
        if lhs != com1 {
            return false;
        }

        let com2 = Commitment::new(&ck, &Scalar::zero(), &zwv.v);
        let lhs = &iba.i * (&cx - &zwv.z) + &iba.a;
        if lhs != com2 {
            return false;
        }
    }

    // check product
    {
        let bits = ciphertexts.bits();
        let cx_pow = cx.power(bits);

        let p1 = ciphertexts
            .as_ref()
            .iter()
            .enumerate()
            .fold(Ciphertext::zero(), |acc, (i, c)| {
                let idx = binrep(i, bits as u32);
                let multz = proof
                    .zwvs
                    .iter()
                    .enumerate()
                    .fold(Scalar::one(), |acc, (j, zwv)| {
                        let m = if idx[j] { zwv.z.clone() } else { &cx - &zwv.z };
                        &acc * m
                    });
                let enc = encrypt(public_key, &multz.negate(), &Scalar::zero());
                let mult_c = c * &cx_pow;
                let y_pow_i = cy.power(i);
                let t = (&mult_c + &enc) * y_pow_i;
                &acc + &t
            });

        let dsum = proof
            .ds
            .iter()
            .enumerate()
            .fold(Ciphertext::zero(), |acc, (l, d)| &acc + &(d * cx.power(l)));

        let zero = encrypt(public_key, &Scalar::zero(), &proof.r);
        if &p1 + &dsum != zero {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encrypted::EncryptingVote;
    use crate::gargamel;
    use crate::unit_vector::UnitVector;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn prove_verify1() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = gargamel::generate(&mut r).public_key;
        let unit_vector = UnitVector::new(2, 0);
        let ev = EncryptingVote::prepare(&mut r, &public_key, &unit_vector);

        let proof = prove(&mut r, &public_key, ev.clone());
        assert!(verify(&public_key, &ev.ciphertexts, &proof))
    }

    #[test]
    fn prove_verify() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = gargamel::generate(&mut r).public_key;
        let unit_vector = UnitVector::new(5, 1);
        let ev = EncryptingVote::prepare(&mut r, &public_key, &unit_vector);

        let proof = prove(&mut r, &public_key, ev.clone());
        assert!(verify(&public_key, &ev.ciphertexts, &proof))
    }
}
