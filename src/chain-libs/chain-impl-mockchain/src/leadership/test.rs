use super::genesis::GenesisPraosLeader;
use super::bft::LeaderId;
use chain_crypto::{testing, Curve25519_2HashDH, PublicKey, SecretKey, SumEd25519_12};
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen, quickcheck, TestResult};
use chain_core::property::testing::serialization_bijection;

impl Arbitrary for GenesisPraosLeader {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        lazy_static! {
            static ref PK_KES: PublicKey<SumEd25519_12> =
                { testing::static_secret_key::<SumEd25519_12>().to_public() };
        }

        let tcg = testing::TestCryptoGen::arbitrary(g);
        let mut rng = tcg.get_rng(0);
        let vrf_sk: SecretKey<Curve25519_2HashDH> = SecretKey::generate(&mut rng);
        GenesisPraosLeader {
            vrf_public_key: vrf_sk.to_public(),
            kes_public_key: PK_KES.clone(),
        }
    }
}

quickcheck! {
    fn leader_id_serialize_deserialize_biyection(leader_id: LeaderId) -> TestResult {
        serialization_bijection(leader_id)
    }
}