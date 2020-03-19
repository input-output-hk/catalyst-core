use super::cstruct;
use chain_crypto::algorithms::vrf::ProvenOutputSeed;
use chain_crypto::{Ed25519, PublicKey, Signature, SumEd25519_12, Verification};
use std::fmt::{self, Debug};

#[derive(Debug, Clone)]
pub struct HeaderAuth;

#[derive(Debug, Clone)]
pub struct KESSignature(pub(crate) Signature<HeaderAuth, SumEd25519_12>);

impl From<cstruct::GpKesSignature> for KESSignature {
    fn from(b: cstruct::GpKesSignature) -> KESSignature {
        KESSignature(
            Signature::from_binary(&b[..]).expect("internal error: KES signature length invalid"),
        )
    }
}

impl From<Signature<HeaderAuth, SumEd25519_12>> for KESSignature {
    fn from(sig: Signature<HeaderAuth, SumEd25519_12>) -> KESSignature {
        KESSignature(sig)
    }
}

impl KESSignature {
    pub fn verify(&self, pk: &PublicKey<SumEd25519_12>, data: &[u8]) -> Verification {
        self.0.verify_slice(pk, data)
    }
}

#[derive(Debug, Clone)]
pub struct BftSignature(pub(crate) Signature<HeaderAuth, Ed25519>);

impl From<cstruct::BftSignature> for BftSignature {
    fn from(b: cstruct::BftSignature) -> BftSignature {
        BftSignature(
            Signature::from_binary(&b[..]).expect("internal error: BFT signature length invalid"),
        )
    }
}

impl From<Signature<HeaderAuth, Ed25519>> for BftSignature {
    fn from(sig: Signature<HeaderAuth, Ed25519>) -> BftSignature {
        BftSignature(sig)
    }
}

#[derive(Clone)]
pub struct VrfProof(pub(super) cstruct::GpVrfProof);

impl Debug for VrfProof {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VrfProof")
            .field("data", &&self.0[..])
            .finish()
    }
}

impl VrfProof {
    pub fn to_vrf_proof(&self) -> Option<ProvenOutputSeed> {
        ProvenOutputSeed::from_bytes_unverified(&self.0)
    }
}

impl From<ProvenOutputSeed> for VrfProof {
    fn from(v: ProvenOutputSeed) -> VrfProof {
        VrfProof(v.bytes())
    }
}
