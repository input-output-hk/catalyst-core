#[cfg(test)]
use crate::block::Header;
#[cfg(test)]
use crate::header::HeaderDesc;
#[cfg(test)]
use crate::testing::serialization::serialization_bijection;
use crate::{
    block::{Block, BlockVersion, HeaderRaw},
    fragment::{Contents, ContentsBuilder, Fragment},
    header::{BftProof, GenesisPraosProof, HeaderBuilderNew},
};
#[cfg(test)]
use chain_core::{
    packer::Codec,
    property::{Block as _, Deserialize, Serialize},
};
#[cfg(test)]
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};

quickcheck! {
    fn headerraw_serialization_bijection(b: HeaderRaw) -> TestResult {
        serialization_bijection(b)
    }
    fn header_serialization_bijection(b: Header) -> TestResult {
        serialization_bijection(b)
    }

    fn block_serialization_bijection(b: Block) -> TestResult {
        serialization_bijection(b)
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

    // TODO: add a separate test with headers with correct content size to stress hash
    // checking when tests are migrated to proptest
    fn inconsistent_block_deserialization(header: Header, contents: Contents) -> bool {
        let (content_hash, content_size) = contents.compute_hash_size();

        let maybe_block = Block { header: header.clone(), contents };

        let block = Block::deserialize(&mut Codec::new(maybe_block.serialize_as_vec().unwrap().as_slice()));

        (content_hash != header.block_content_hash() || content_size != header.block_content_size()) == block.is_err()
    }
}

#[cfg(test)]
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
