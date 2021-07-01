mod dleq;
#[allow(clippy::module_inception)]
pub mod vrf;

use crate::key::{
    AsymmetricKey, AsymmetricPublicKey, PublicKeyError, SecretKeyError, SecretKeySizeStatic,
};
use crate::vrf::{VerifiableRandomFunction, VrfVerification};
use rand_core::{CryptoRng, RngCore};

pub use vrf::ProvenOutputSeed;

/// VRF
pub struct EllipticCurve2hashDh;

impl AsymmetricPublicKey for EllipticCurve2hashDh {
    type Public = vrf::PublicKey;
    const PUBLIC_BECH32_HRP: &'static str = "vrf_pk";
    const PUBLIC_KEY_SIZE: usize = vrf::PublicKey::BYTES_LEN;
    fn public_from_binary(data: &[u8]) -> Result<Self::Public, PublicKeyError> {
        vrf::PublicKey::from_bytes(data)
    }
}

impl AsymmetricKey for EllipticCurve2hashDh {
    type Secret = vrf::SecretKey;
    type PubAlg = EllipticCurve2hashDh;

    const SECRET_BECH32_HRP: &'static str = "vrf_sk";

    fn generate<T: RngCore + CryptoRng>(rng: T) -> Self::Secret {
        Self::Secret::random(rng)
    }

    fn compute_public(key: &Self::Secret) -> <Self::PubAlg as AsymmetricPublicKey>::Public {
        key.public()
    }

    fn secret_from_binary(data: &[u8]) -> Result<Self::Secret, SecretKeyError> {
        if data.len() != vrf::SecretKey::BYTES_LEN {
            return Err(SecretKeyError::SizeInvalid);
        }
        let mut buf = [0; vrf::SecretKey::BYTES_LEN];
        buf[0..vrf::SecretKey::BYTES_LEN].clone_from_slice(data);
        match vrf::SecretKey::from_bytes(buf) {
            None => Err(SecretKeyError::StructureInvalid),
            Some(k) => Ok(k),
        }
    }
}

impl SecretKeySizeStatic for EllipticCurve2hashDh {
    const SECRET_KEY_SIZE: usize = vrf::SecretKey::BYTES_LEN;
}

impl VerifiableRandomFunction for EllipticCurve2hashDh {
    type VerifiedRandomOutput = vrf::ProvenOutputSeed;
    type RandomOutput = vrf::OutputSeed;
    type Input = [u8];

    const VERIFIED_RANDOM_SIZE: usize = vrf::ProvenOutputSeed::BYTES_LEN;

    fn evaluate_and_prove<T: RngCore + CryptoRng>(
        secret: &Self::Secret,
        input: &Self::Input,
        mut rng: T,
    ) -> Self::VerifiedRandomOutput {
        secret.evaluate_simple(&mut rng, input)
    }

    fn verify(
        public: &Self::Public,
        input: &Self::Input,
        vrand: &Self::VerifiedRandomOutput,
    ) -> VrfVerification {
        let v = vrand.verify(public, input);
        if v {
            VrfVerification::Success
        } else {
            VrfVerification::Failed
        }
    }

    fn strip_verification_output(vr: &Self::VerifiedRandomOutput) -> Self::RandomOutput {
        vr.u.clone()
    }
}
