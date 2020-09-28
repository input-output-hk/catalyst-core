#[cfg(test)]
use crate::block::Header;
use crate::{
    block::{Block, BlockVersion, HeaderRaw},
    fragment::{Contents, ContentsBuilder, Fragment},
    header::{BftProof, GenesisPraosProof, HeaderBuilderNew, HeaderDesc},
};
use chain_core::property::{Block as _, Deserialize, HasHeader as _, Serialize};
#[cfg(test)]
use chain_test_utils::property;
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};

quickcheck! {
    fn headerraw_serialization_bijection(b: HeaderRaw) -> TestResult {
        property::serialization_bijection(b)
    }

    fn header_serialization_bijection(b: Header) -> TestResult {
        property::serialization_bijection_r(b)
    }

    fn block_serialization_bijection(b: Block) -> TestResult {
        property::serialization_bijection(b)
    }

    fn block_serialization_bijection_r(b: Block) -> TestResult {
        property::serialization_bijection_r(b)
    }

    fn block_properties(block: Block) -> TestResult {

        let vec = block.serialize_as_vec().unwrap();
        let new_block = Block::deserialize(&vec[..]).unwrap();

        assert_eq!(block.is_consistent(),new_block.is_consistent());
        assert!(block.fragments().eq(new_block.fragments()));
        assert_eq!(block.header(),new_block.header());
        assert_eq!(block.id(),new_block.id());
        assert_eq!(block.parent_id(),new_block.parent_id());
        assert_eq!(block.date(),new_block.date());
        assert_eq!(block.version(),new_block.version());

        TestResult::from_bool(block.chain_length() == new_block.chain_length())
    }

    fn header_properties(block: Block) -> TestResult {
        use chain_core::property::Header as Prop;
        let header = block.header.clone();
        assert_eq!(header.hash(),block.id());
        assert_eq!(header.id(),block.id());
        assert_eq!(header.parent_id(),block.parent_id());
        assert_eq!(header.date(),block.date());
        assert_eq!(header.version(),block.version());

        assert_eq!(header.get_bft_leader_id(),block.header.get_bft_leader_id());
        assert_eq!(header.get_stakepool_id(),block.header.get_stakepool_id());
        assert_eq!(header.common(),block.header.common());
        assert_eq!(header.to_raw(),block.header.to_raw());
        assert_eq!(header.as_auth_slice(),block.header.as_auth_slice());
        assert!(are_desc_equal(header.description(),block.header.description()));
        assert_eq!(header.size(),block.header.size());

        TestResult::from_bool(header.chain_length() == block.chain_length())
    }
}

fn are_desc_equal(left: HeaderDesc, right: HeaderDesc) -> bool {
    left.id == right.id
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
            BlockVersion::Genesis => hdrbuilder.into_unsigned_header().unwrap().generalize(),
            BlockVersion::Ed25519Signed => {
                let bft_proof: BftProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .into_bft_builder()
                    .unwrap()
                    .set_consensus_data(&bft_proof.leader_id)
                    .set_signature(bft_proof.signature)
                    .generalize()
            }
            BlockVersion::KesVrfproof => {
                let gp_proof: GenesisPraosProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .into_genesis_praos_builder()
                    .unwrap()
                    .set_consensus_data(&gp_proof.node_id, &gp_proof.vrf_proof)
                    .set_signature(gp_proof.kes_proof)
                    .generalize()
            }
        };
        Block {
            header,
            contents: content,
        }
    }
}
