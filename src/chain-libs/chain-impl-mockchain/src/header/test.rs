use super::*;
use crate::header::{BftProof, BftSignature, Common, GenesisPraosProof, KESSignature};
use crate::leadership;
use chain_core::property;
use chain_crypto::{
    self, AsymmetricKey, Curve25519_2HashDH, Ed25519, SecretKey, SumEd25519_12,
    VerifiableRandomFunction,
};
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen, TestResult};
quickcheck! {
    fn header_serialization_bijection(b: Header) -> TestResult {
        property::testing::serialization_bijection_r(b)
    }
}

impl Arbitrary for BlockVersion {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        BlockVersion::from_u16(u16::arbitrary(g) % 3).unwrap()
    }
}

impl Arbitrary for Common {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Common {
            block_version: Arbitrary::arbitrary(g),
            block_date: Arbitrary::arbitrary(g),
            block_content_size: Arbitrary::arbitrary(g),
            block_content_hash: Arbitrary::arbitrary(g),
            block_parent_hash: Arbitrary::arbitrary(g),
            chain_length: ChainLength(Arbitrary::arbitrary(g)),
        }
    }
}

impl Arbitrary for BftProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let sk: chain_crypto::SecretKey<Ed25519> = Arbitrary::arbitrary(g);
        let pk = sk.to_public();
        let signature = sk.sign(&[0u8, 1, 2, 3]);
        BftProof {
            leader_id: leadership::bft::LeaderId(pk),
            signature: BftSignature(signature.coerce()),
        }
    }
}
impl Arbitrary for GenesisPraosProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        use rand_chacha::ChaChaRng;
        use rand_core::SeedableRng;
        let mut seed = [0; 32];
        for byte in seed.iter_mut() {
            *byte = Arbitrary::arbitrary(g);
        }
        let mut rng = ChaChaRng::from_seed(seed);

        let node_id = Arbitrary::arbitrary(g);

        let vrf_proof = {
            let sk = Curve25519_2HashDH::generate(&mut rng);
            Curve25519_2HashDH::evaluate_and_prove(&sk, &[0, 1, 2, 3], &mut rng)
        };

        let kes_proof = {
            lazy_static! {
                static ref SK_FIRST: SecretKey<SumEd25519_12> =
                    { SecretKey::generate(&mut ChaChaRng::from_seed([0; 32])) };
            }
            let sk = SK_FIRST.clone();
            let signature = sk.sign(&[0u8, 1, 2, 3]);
            KESSignature(signature.coerce())
        };
        GenesisPraosProof {
            node_id: node_id,
            vrf_proof: vrf_proof.into(),
            kes_proof: kes_proof,
        }
    }
}

impl Arbitrary for Header {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let common = Common::arbitrary(g);
        let hdrbuilder = HeaderBuilderNew::new_raw(
            common.block_version,
            &common.block_content_hash,
            common.block_content_size,
        )
        .set_parent(&common.block_parent_hash, common.chain_length)
        .set_date(common.block_date);
        let header = match common.block_version {
            BlockVersion::Genesis => hdrbuilder.to_unsigned_header().unwrap().generalize(),
            BlockVersion::Ed25519Signed => {
                let bft_proof: BftProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .to_bft_builder()
                    .unwrap()
                    .set_consensus_data(&bft_proof.leader_id)
                    .set_signature(bft_proof.signature)
                    .generalize()
            }
            BlockVersion::KesVrfproof => {
                let gp_proof: GenesisPraosProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .to_genesis_praos_builder()
                    .unwrap()
                    .set_consensus_data(&gp_proof.node_id, &gp_proof.vrf_proof.into())
                    .set_signature(gp_proof.kes_proof)
                    .generalize()
            }
        };

        header
    }
}

impl Arbitrary for ChainLength {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        ChainLength(Arbitrary::arbitrary(g))
    }
}
