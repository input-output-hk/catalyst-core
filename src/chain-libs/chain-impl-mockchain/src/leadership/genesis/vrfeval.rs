/// This contains the current evaluation methods for the VRF and its link to
/// the stake distribution
use crate::chaineval::PraosNonce;
use crate::date::SlotId;
use crate::setting::ActiveSlotsCoeff;
use crate::stake::PercentStake;
use chain_crypto::{
    vrf_evaluate_and_prove, vrf_verified_get_output, vrf_verify, Curve25519_2HashDH, PublicKey,
    SecretKey, VRFVerification, VerifiableRandomFunction,
};
use rand_core::OsRng;

/// Threshold between 0.0 and 1.0
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Threshold(f64);

impl Threshold {
    pub fn from_u256(v: &[u8]) -> Self {
        assert_eq!(v.len(), 32);
        // TODO, only consider the highest part
        let v64 = (v[0] as u64) << 56
            | (v[1] as u64) << 48
            | (v[2] as u64) << 40
            | (v[3] as u64) << 32
            | (v[4] as u64) << 24
            | (v[5] as u64) << 16
            | (v[6] as u64) << 8
            | (v[7] as u64);
        Threshold((v64 as f64) / 18_446_744_073_709_551_616.0)
    }
}

/// previous epoch nonce and the slotid encoded in big endian
struct Input([u8; 36]);

impl Input {
    /// Create an Input from previous epoch nonce and the current slotid
    fn create(epoch_nonce: &PraosNonce, slotid: SlotId) -> Self {
        let mut input = [0u8; 36];
        input[0..32].copy_from_slice(epoch_nonce.as_ref());
        input[32..].copy_from_slice(&slotid.to_le_bytes());
        Input(input)
    }
}

/// Witness
pub type Witness = <Curve25519_2HashDH as VerifiableRandomFunction>::VerifiedRandomOutput;
pub type WitnessOutput = <Curve25519_2HashDH as VerifiableRandomFunction>::RandomOutput;

pub struct VrfEvaluator<'a> {
    pub stake: PercentStake,
    pub nonce: &'a PraosNonce,
    pub slot_id: SlotId,
    pub active_slots_coeff: ActiveSlotsCoeff,
}

pub(crate) fn witness_to_nonce(witness: &Witness) -> PraosNonce {
    let r = vrf_verified_get_output::<Curve25519_2HashDH>(&witness);
    get_nonce(&r)
}

#[derive(Clone, Debug)]
pub enum VrfEvalFailure {
    ProofVerificationFailed,
    ThresholdNotMet {
        vrf_value: f64,
        stake_threshold: f64,
    },
}

impl<'a> VrfEvaluator<'a> {
    /// Evaluate if the threshold is above for a given input for the key and the associated stake
    ///
    /// On threshold success, the witness is returned, otherwise None is returned
    pub fn evaluate(&self, key: &SecretKey<Curve25519_2HashDH>) -> Option<Witness> {
        let input = Input::create(self.nonce, self.slot_id);
        let csprng = OsRng;
        let vr = vrf_evaluate_and_prove(key, &input.0, csprng);
        let r = vrf_verified_get_output::<Curve25519_2HashDH>(&vr);
        let t = get_threshold(&input, &r);
        if above_stake_threshold(t, &self.stake, self.active_slots_coeff) {
            Some(vr)
        } else {
            None
        }
    }

    /// verify that the witness pass the threshold for this witness for a given
    /// key and its associated stake.
    ///
    /// On success, the nonce is returned, otherwise None is returned
    pub fn verify(
        &self,
        key: &PublicKey<Curve25519_2HashDH>,
        witness: &'a Witness,
    ) -> Result<PraosNonce, VrfEvalFailure> {
        let input = Input::create(&self.nonce, self.slot_id);
        if vrf_verify(key, &input.0, witness) == VRFVerification::Success {
            let r = vrf_verified_get_output::<Curve25519_2HashDH>(witness);
            // compare threshold against phi-adjusted-stake
            let threshold = get_threshold(&input, &r);
            let phi_stake = phi(self.active_slots_coeff, &self.stake);
            if threshold < phi_stake {
                Ok(get_nonce(&r))
            } else {
                Err(VrfEvalFailure::ThresholdNotMet {
                    vrf_value: threshold.0,
                    stake_threshold: phi_stake.0,
                })
            }
        } else {
            Err(VrfEvalFailure::ProofVerificationFailed)
        }
    }
}

fn above_stake_threshold(
    threshold: Threshold,
    stake: &PercentStake,
    active_slots_coeff: ActiveSlotsCoeff,
) -> bool {
    threshold < phi(active_slots_coeff, stake)
}

fn phi(active_slots_coeff: ActiveSlotsCoeff, rs: &PercentStake) -> Threshold {
    let t = rs.as_float();
    let f: f64 = active_slots_coeff.into();
    Threshold(1.0 - (1.0 - f).powf(t))
}

const DOMAIN_NONCE: &[u8] = b"NONCE";
const DOMAIN_THRESHOLD: &[u8] = b"TEST";

fn get_threshold(input: &Input, os: &WitnessOutput) -> Threshold {
    let out = os.to_output(&input.0, DOMAIN_THRESHOLD);
    // read as big endian 64 bits values from left to right.
    Threshold::from_u256(out.as_ref())
}

fn get_nonce(os: &WitnessOutput) -> PraosNonce {
    let mut nonce = [0u8; 32];
    let out = os.to_output(&[], DOMAIN_NONCE);
    nonce.copy_from_slice(out.as_ref());
    PraosNonce::from_output_array(nonce)
}
