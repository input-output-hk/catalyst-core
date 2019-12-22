#[warn(unused_imports)]
use crate::{
    block::{Block, BlockVersion, ConsensusVersion, Header, HeaderRaw},
    fragment::{Contents, ContentsBuilder, Fragment},
    header::{BftProof, GenesisPraosProof, HeaderBuilderNew},
};
use chain_core::property;
use quickcheck::{Arbitrary, Gen, TestResult};

impl Arbitrary for ConsensusVersion {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        ConsensusVersion::from_u16(u16::arbitrary(g) % 2 + 1).unwrap()
    }
}

quickcheck! {
    fn headerraw_serialization_bijection(b: HeaderRaw) -> TestResult {
        property::testing::serialization_bijection(b)
    }

    fn header_serialization_bijection(b: Header) -> TestResult {
        property::testing::serialization_bijection_r(b)
    }

    fn block_serialization_bijection(b: Block) -> TestResult {
        property::testing::serialization_bijection(b)
    }
}

impl Arbitrary for HeaderRaw {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let len = u16::arbitrary(g);
        let mut v = Vec::new();
        for _ in 0..len {
            v.push(u8::arbitrary(g))
        }
        HeaderRaw(v)
    }
}

impl Arbitrary for Contents {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let len = u8::arbitrary(g) % 12;
        let fragments: Vec<Fragment> = std::iter::repeat_with(|| Arbitrary::arbitrary(g))
            .take(len as usize)
            .collect();
        let mut content = ContentsBuilder::new();
        content.push_many(fragments);
        content.into()
    }
}

impl Arbitrary for Block {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let content = Contents::arbitrary(g);
        let ver = BlockVersion::arbitrary(g);
        let parent_hash = Arbitrary::arbitrary(g);
        let chain_length = Arbitrary::arbitrary(g);
        let date = Arbitrary::arbitrary(g);
        let hdrbuilder = HeaderBuilderNew::new(ver, &content)
            .set_parent(&parent_hash, chain_length)
            .set_date(date);
        let header = match ver {
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
        Block {
            header: header,
            contents: content,
        }
    }
}
