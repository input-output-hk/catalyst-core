//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! ["A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence"](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//! We use the notation presented in the technical
//! [spec](https://github.com/input-output-hk/treasury-crypto/blob/master/docs/voting_protocol_spec/Treasury_voting_protocol_spec.pdf),
//! written by Dmytro Kaidalov.

use chain_core::mempack::{ReadBuf, ReadError};
use chain_crypto::ec::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "ristretto255")]
use {rand::thread_rng, std::iter};

use super::challenge_context::ChallengeContext;
use super::messages::{generate_polys, Announcement, BlindingRandomness, ResponseRandomness};
use crate::cryptography::CommitmentKey;
#[cfg(not(feature = "ristretto255"))]
use crate::cryptography::Open;
use crate::cryptography::{Ciphertext, PublicKey};
use crate::encrypted_vote::{binrep, Ptp, UnitVector};
use crate::tally::Crs;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Zkp {
    /// Commitment to the proof randomness and bits of binary representaion of `i`
    ibas: Vec<Announcement>,
    /// Encryption to the polynomial coefficients used in the proof
    ds: Vec<Ciphertext>,
    /// Response related to the randomness committed in `ibas`
    zwvs: Vec<ResponseRandomness>,
    /// Final response
    r: Scalar,
}

#[allow(clippy::len_without_is_empty)]
impl Zkp {
    /// Generate a unit vector proof. In this proof, a prover encrypts each entry of a
    /// vector `unit_vector`, and proves
    /// that the vector is a unit vector. In particular, it proves that it is the `i`th unit
    /// vector without disclosing `i`.
    /// Common Reference String (`Crs`): Pedersen Commitment Key
    /// Statement: public key `pk`, and ciphertexts `ciphertexts`
    /// C_0=Enc_pk(r_0; v_0), ..., C_{m-1}=Enc_pk(r_{m-1}; v_{m-1})
    /// Witness: the unit vector `unit_vector`, and randomness used for
    /// encryption `encryption_randomness`.
    ///
    /// The proof communication complexity is logarithmic with respect to the size of
    /// the encrypted tuple. Description of the proof available in Figure 8.
    pub(crate) fn generate<R: RngCore + CryptoRng>(
        rng: &mut R,
        crs: &Crs,
        public_key: &PublicKey,
        unit_vector: &UnitVector,
        encryption_randomness: &[Scalar],
        ciphertexts: &[Ciphertext],
    ) -> Self {
        let ck = CommitmentKey::from(crs.clone());
        let ciphers = Ptp::new(ciphertexts.to_vec(), Ciphertext::zero);
        let cipher_randoms = Ptp::new(encryption_randomness.to_vec(), Scalar::zero);

        assert_eq!(ciphers.bits(), cipher_randoms.bits());

        let bits = ciphers.bits();

        let mut blinding_randomness_vec = Vec::with_capacity(bits);
        let mut first_announcement_vec = Vec::with_capacity(bits);
        let idx_binary_rep = binrep(unit_vector.ith(), bits as u32);
        for &i in idx_binary_rep.iter() {
            let (b_rand, ann) = BlindingRandomness::gen_and_commit(&ck, i, rng);
            blinding_randomness_vec.push(b_rand);
            first_announcement_vec.push(ann);
        }

        // Generate First verifier challenge
        let mut cc = ChallengeContext::new(&ck, public_key, ciphers.as_ref());
        let cy = cc.first_challenge(&first_announcement_vec);

        let (poly_coeff_enc, rs) = {
            let pjs = generate_polys(
                ciphers.len(),
                &idx_binary_rep,
                bits,
                &blinding_randomness_vec,
            );

            // Generate new Rs for Ds
            let mut rs = Vec::with_capacity(bits);
            let mut ds = Vec::with_capacity(bits);

            for i in 0..bits {
                let sum =
                    cy.exp_iter()
                        .zip(pjs.iter())
                        .fold(Scalar::zero(), |sum, (c_pows, pj)| {
                            let s = sum + c_pows * pj.get_coefficient_at(i);
                            s
                        });

                let (d, r) = public_key.encrypt_return_r(&sum, rng);
                ds.push(d);
                rs.push(r);
            }
            (ds, rs)
        };

        // Generate second verifier challenge
        let cx = cc.second_challenge(&poly_coeff_enc);

        // Compute ZWVs
        let randomness_response_vec = blinding_randomness_vec
            .iter()
            .zip(idx_binary_rep.iter())
            .map(|(abcd, index)| abcd.gen_response(&cx, index))
            .collect::<Vec<_>>();

        // Compute R
        let response = {
            let cx_pow = cx.power(cipher_randoms.bits());
            let p1 = cipher_randoms.iter().zip(cy.exp_iter()).fold(
                Scalar::zero(),
                |acc, (r, cy_pows)| {
                    let el = r * &cx_pow * cy_pows;
                    el + acc
                },
            );
            let p2 = rs
                .iter()
                .zip(cx.exp_iter())
                .fold(Scalar::zero(), |acc, (r, cx_pows)| {
                    let el = r * cx_pows;
                    el + acc
                });
            p1 + p2
        };

        Zkp {
            ibas: first_announcement_vec,
            ds: poly_coeff_enc,
            zwvs: randomness_response_vec,
            r: response,
        }
    }

    /// Verify a unit vector proof. The verifier checks that the plaintexts encrypted in `ciphertexts`,
    /// under `public_key` represent a unit vector.
    /// Common Reference String (`crs`): Pedersen Commitment Key
    /// Statement: public key `pk`, and ciphertexts `ciphertexts`
    /// C_0=Enc_pk(r_0; v_0), ..., C_{m-1}=Enc_pk(r_{m-1}; v_{m-1})
    ///
    /// Description of the verification procedure available in Figure 9.
    pub fn verify(&self, crs: &Crs, public_key: &PublicKey, ciphertexts: &[Ciphertext]) -> bool {
        let ck = CommitmentKey::from(crs.clone());
        let ciphertexts = Ptp::new(ciphertexts.to_vec(), Ciphertext::zero);
        let bits = ciphertexts.bits();
        let mut cc = ChallengeContext::new(&ck, public_key, ciphertexts.as_ref());
        let cy = cc.first_challenge(&self.ibas);
        let cx = cc.second_challenge(&self.ds);

        if self.ibas.len() != bits {
            return false;
        }

        if self.zwvs.len() != bits {
            return false;
        }

        self.verify_statements(public_key, &ck, &ciphertexts, &cx, &cy)
    }

    /// Final verification of the proof, that we compute in a single vartime multiscalar
    /// multiplication.
    #[cfg(feature = "ristretto255")]
    fn verify_statements(
        &self,
        public_key: &PublicKey,
        commitment_key: &CommitmentKey,
        ciphertexts: &Ptp<Ciphertext>,
        challenge_x: &Scalar,
        challenge_y: &Scalar,
    ) -> bool {
        let bits = ciphertexts.bits();
        let length = ciphertexts.len();
        let cx_pow = challenge_x.power(bits);

        let powers_cx = challenge_x.exp_iter();
        let powers_cy = challenge_y.exp_iter();

        let powers_z_iterator = powers_z_encs_iter(&self.zwvs, challenge_x, &(bits as u32));

        let zero = public_key.encrypt_with_r(&Scalar::zero(), &self.r);

        // Challenge value for batching two equations into a single multiscalar mult.
        let batch_challenge = Scalar::random(&mut thread_rng());

        for (zwv, iba) in self.zwvs.iter().zip(self.ibas.iter()) {
            if GroupElement::multiscalar_multiplication(
                iter::once(zwv.z)
                    .chain(iter::once(zwv.w + batch_challenge * zwv.v))
                    .chain(iter::once(
                        batch_challenge * (zwv.z - challenge_x) - challenge_x,
                    ))
                    .chain(iter::once(Scalar::one().negate()))
                    .chain(iter::once(batch_challenge.negate())),
                iter::once(GroupElement::generator())
                    .chain(iter::once(commitment_key.h))
                    .chain(iter::once(iba.i))
                    .chain(iter::once(iba.b))
                    .chain(iter::once(iba.a)),
            ) != GroupElement::zero()
            {
                return false;
            }
        }

        let mega_check = GroupElement::multiscalar_multiplication(
            powers_cy
                .take(length)
                .map(|s| s * cx_pow)
                .chain(powers_cy.take(length).map(|s| s * cx_pow))
                .chain(powers_cy.take(length))
                .chain(powers_cx.take(bits))
                .chain(powers_cx.take(bits))
                .chain(iter::once(Scalar::one().negate()))
                .chain(iter::once(Scalar::one().negate())),
            ciphertexts
                .iter()
                .map(|ctxt| ctxt.e2)
                .chain(ciphertexts.iter().map(|ctxt| ctxt.e1))
                .chain(powers_z_iterator.take(length))
                .chain(self.ds.iter().map(|ctxt| ctxt.e1))
                .chain(self.ds.iter().map(|ctxt| ctxt.e2))
                .chain(iter::once(zero.e1))
                .chain(iter::once(zero.e2)),
        );

        mega_check == GroupElement::zero()
    }

    // Final verification of the proof. We do not use the multiscalar optimisation when using sec2 curves.
    #[cfg(not(feature = "ristretto255"))]
    fn verify_statements(
        &self,
        public_key: &PublicKey,
        commitment_key: &CommitmentKey,
        ciphertexts: &Ptp<Ciphertext>,
        challenge_x: &Scalar,
        challenge_y: &Scalar,
    ) -> bool {
        // check commitments are 0 / 1
        for (iba, zwv) in self.ibas.iter().zip(self.zwvs.iter()) {
            if !commitment_key.verify(
                &(&iba.i * challenge_x + &iba.b),
                &Open {
                    m: zwv.z.clone(),
                    r: zwv.w.clone(),
                },
            ) {
                return false;
            }

            if !commitment_key.verify(
                &(&iba.i * (challenge_x - &zwv.z) + &iba.a),
                &Open {
                    m: Scalar::zero(),
                    r: zwv.v.clone(),
                },
            ) {
                return false;
            }
        }

        let bits = ciphertexts.bits();
        let cx_pow = challenge_x.power(bits);

        let p1 = ciphertexts
            .as_ref()
            .iter()
            .zip(challenge_y.exp_iter())
            .enumerate()
            .fold(Ciphertext::zero(), |acc, (i, (c, cy_pows))| {
                let multz = powers_z_encs(&self.zwvs, challenge_x.clone(), i, bits as u32);
                let enc = public_key.encrypt_with_r(&multz.negate(), &Scalar::zero());
                let mult_c = c * &cx_pow;
                let t = (&mult_c + &enc) * cy_pows;
                &acc + &t
            });

        let dsum = self
            .ds
            .iter()
            .zip(challenge_x.exp_iter())
            .fold(Ciphertext::zero(), |acc, (d, cx_pows)| {
                &acc + &(d * cx_pows)
            });

        let zero = public_key.encrypt_with_r(&Scalar::zero(), self.r());

        &p1 + &dsum - zero == Ciphertext::zero()
    }

    /// Try to generate a `Proof` from a buffer
    pub fn from_buffer(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        let bits = buf.get_u8()? as usize;
        let mut ibas = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(Announcement::BYTES_LEN)?;
            let iba = Announcement::from_bytes(elem_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid IBA component".to_string()))?;
            ibas.push(iba);
        }
        let mut bs = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(Ciphertext::BYTES_LEN)?;
            let ciphertext = Ciphertext::from_bytes(elem_buf).ok_or_else(|| {
                ReadError::StructureInvalid("Invalid encoded ciphertext".to_string())
            })?;
            bs.push(ciphertext);
        }
        let mut zwvs = Vec::with_capacity(bits);
        for _ in 0..bits {
            let elem_buf = buf.get_slice(ResponseRandomness::BYTES_LEN)?;
            let zwv = ResponseRandomness::from_bytes(elem_buf)
                .ok_or_else(|| ReadError::StructureInvalid("Invalid ZWV component".to_string()))?;
            zwvs.push(zwv);
        }
        let r_buf = buf.get_slice(Scalar::BYTES_LEN)?;
        let r = Scalar::from_bytes(r_buf).ok_or_else(|| {
            ReadError::StructureInvalid("Invalid Proof encoded R scalar".to_string())
        })?;

        Ok(Self::from_parts(ibas, bs, zwvs, r))
    }

    /// Constructs the proof structure from constituent parts.
    ///
    /// # Panics
    ///
    /// The `ibas`, `ds`, and `zwvs` must have the same length, otherwise the function will panic.
    pub fn from_parts(
        ibas: Vec<Announcement>,
        ds: Vec<Ciphertext>,
        zwvs: Vec<ResponseRandomness>,
        r: Scalar,
    ) -> Self {
        assert_eq!(ibas.len(), ds.len());
        assert_eq!(ibas.len(), zwvs.len());
        Zkp { ibas, ds, zwvs, r }
    }

    /// Returns the length of the size of the witness vector
    pub fn len(&self) -> usize {
        self.ibas.len()
    }

    /// Return an iterator of the announcement commitments
    pub fn ibas(&self) -> impl Iterator<Item = &Announcement> {
        self.ibas.iter()
    }

    /// Return an iterator of the encryptions of the polynomial coefficients
    pub fn ds(&self) -> impl Iterator<Item = &Ciphertext> {
        self.ds.iter()
    }

    /// Return an iterator of the response related to the randomness
    pub fn zwvs(&self) -> impl Iterator<Item = &ResponseRandomness> {
        self.zwvs.iter()
    }

    /// Return R
    pub fn r(&self) -> &Scalar {
        &self.r
    }
}

// Computes the product of the powers of `z` given the `challenge_x`, `index` and a `bit_size`
fn powers_z_encs(
    z: &[ResponseRandomness],
    challenge_x: Scalar,
    index: usize,
    bit_size: u32,
) -> Scalar {
    let idx = binrep(index, bit_size as u32);

    let multz = z.iter().enumerate().fold(Scalar::one(), |acc, (j, zwv)| {
        let m = if idx[j] {
            zwv.z.clone()
        } else {
            &challenge_x - &zwv.z
        };
        &acc * m
    });
    multz
}

/// Provides an iterator over the encryptions of the product of the powers of `z`.
///
/// This struct is created by the `powers_z_encs_iter` function.
struct ZPowExp {
    index: usize,
    bit_size: u32,
    z: Vec<ResponseRandomness>,
    challenge_x: Scalar,
}

impl Iterator for ZPowExp {
    type Item = GroupElement;

    fn next(&mut self) -> Option<GroupElement> {
        let z_pow = powers_z_encs(&self.z, self.challenge_x.clone(), self.index, self.bit_size);
        self.index += 1;
        Some(z_pow.negate() * GroupElement::generator())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

// Return an iterator of the powers of `ZPowExp`.
#[allow(dead_code)] // can be removed if the default flag is ristretto instead of sec2
fn powers_z_encs_iter(z: &[ResponseRandomness], challenge_x: &Scalar, bit_size: &u32) -> ZPowExp {
    ZPowExp {
        index: 0,
        bit_size: *bit_size,
        z: z.to_vec(),
        challenge_x: challenge_x.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn prove_verify1() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = PublicKey {
            pk: GroupElement::from_hash(&[1u8]),
        };
        let unit_vector = UnitVector::new(2, 0);
        let encryption_randomness = vec![Scalar::random(&mut r); unit_vector.len()];
        let ciphertexts: Vec<Ciphertext> = unit_vector
            .iter()
            .zip(encryption_randomness.iter())
            .map(|(i, r)| public_key.encrypt_with_r(&Scalar::from(i), r))
            .collect();

        let mut shared_string =
            b"Example of a shared string. This could be the latest block hash".to_owned();
        let crs = Crs::from_hash(&mut shared_string);

        let proof = Zkp::generate(
            &mut r,
            &crs,
            &public_key,
            &unit_vector,
            &encryption_randomness,
            &ciphertexts,
        );
        assert!(proof.verify(&crs, &public_key, &ciphertexts))
    }

    #[test]
    fn prove_verify() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = PublicKey {
            pk: GroupElement::from_hash(&[1u8]),
        };
        let unit_vector = UnitVector::new(2, 0);
        let encryption_randomness = vec![Scalar::random(&mut r); unit_vector.len()];
        let ciphertexts: Vec<Ciphertext> = unit_vector
            .iter()
            .zip(encryption_randomness.iter())
            .map(|(i, r)| public_key.encrypt_with_r(&Scalar::from(i), r))
            .collect();

        let mut shared_string =
            b"Example of a shared string. This could be the latest block hash".to_owned();
        let crs = Crs::from_hash(&mut shared_string);

        let proof = Zkp::generate(
            &mut r,
            &crs,
            &public_key,
            &unit_vector,
            &encryption_randomness,
            &ciphertexts,
        );
        assert!(proof.verify(&crs, &public_key, &ciphertexts))
    }

    #[test]
    fn false_proof() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = PublicKey {
            pk: GroupElement::from_hash(&[1u8]),
        };
        let unit_vector = UnitVector::new(2, 0);
        let encryption_randomness = vec![Scalar::random(&mut r); unit_vector.len()];
        let ciphertexts: Vec<Ciphertext> = unit_vector
            .iter()
            .zip(encryption_randomness.iter())
            .map(|(i, r)| public_key.encrypt_with_r(&Scalar::from(i), r))
            .collect();

        let mut shared_string =
            b"Example of a shared string. This could be the latest block hash".to_owned();
        let crs = Crs::from_hash(&mut shared_string);

        let proof = Zkp::generate(
            &mut r,
            &crs,
            &public_key,
            &unit_vector,
            &encryption_randomness,
            &ciphertexts,
        );

        let fake_encryption = [
            Ciphertext::zero(),
            Ciphertext::zero(),
            Ciphertext::zero(),
            Ciphertext::zero(),
            Ciphertext::zero(),
        ];
        assert!(!proof.verify(&crs, &public_key, &fake_encryption))
    }

    #[test]
    fn challenge_context() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);
        let public_key = PublicKey {
            pk: GroupElement::from_hash(&[1u8]),
        };
        let unit_vector = UnitVector::new(2, 0);
        let encryption_randomness = vec![Scalar::random(&mut r); unit_vector.len()];
        let ciphertexts: Vec<Ciphertext> = unit_vector
            .iter()
            .zip(encryption_randomness.iter())
            .map(|(i, r)| public_key.encrypt_with_r(&Scalar::from(i), r))
            .collect();

        let crs = GroupElement::from_hash(&[0u8]);
        let ck = CommitmentKey::from(crs.clone());

        let proof = Zkp::generate(
            &mut r,
            &crs,
            &public_key,
            &unit_vector,
            &encryption_randomness,
            &ciphertexts,
        );

        let mut cc1 = ChallengeContext::new(&ck, &public_key, &ciphertexts);
        let cy1 = cc1.first_challenge(&proof.ibas);
        let cx1 = cc1.second_challenge(&proof.ds);

        // if we set up a new challenge context, the results should be equal
        let mut cc2 = ChallengeContext::new(&ck, &public_key, &ciphertexts);
        let cy2 = cc2.first_challenge(&proof.ibas);
        let cx2 = cc2.second_challenge(&proof.ds);

        assert_eq!(cy1, cy2);
        assert_eq!(cx1, cx2);

        // if we set up a new challenge with incorrect initialisation, results should differ
        let crs_diff = GroupElement::from_hash(&[1u8]);
        let ck_diff = CommitmentKey::from(crs_diff.clone());
        let mut cc3 = ChallengeContext::new(&ck_diff, &public_key, &ciphertexts);
        let cy3 = cc3.first_challenge(&proof.ibas);
        let cx3 = cc3.second_challenge(&proof.ds);

        assert_ne!(cy1, cy3);
        assert_ne!(cx1, cx3);

        // if we generate a new challenge with different IBAs, but same Ds, both results should differ
        let proof_diff = Zkp::generate(
            &mut r,
            &crs,
            &public_key,
            &unit_vector,
            &encryption_randomness,
            &ciphertexts,
        );
        let mut cc4 = ChallengeContext::new(&ck, &public_key, &ciphertexts);
        let cy4 = cc4.first_challenge(&proof_diff.ibas);
        let cx4 = cc4.second_challenge(&proof.ds);

        assert_ne!(cy1, cy4);
        assert_ne!(cx1, cx4);

        // if we generate a challenge with different Ds, only the second scalar should differ
        let mut cc5 = ChallengeContext::new(&ck, &public_key, &ciphertexts);
        let cy5 = cc5.first_challenge(&proof.ibas);
        let cx5 = cc5.second_challenge(&proof_diff.ds);

        assert_eq!(cy1, cy5);
        assert_ne!(cx1, cx5);
    }
}
