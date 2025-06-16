use crate::cardano_node::Settings;
use cardano_serialization_lib::{
    AuxiliaryData, AuxiliaryDataSet, BigNum, Block, BlockHash, Ed25519Signature, Header,
    HeaderBody, KESSignature, KESVKey, OperationalCert, PrivateKey, ProtocolVersion,
    Transaction, TransactionBodies, TransactionWitnessSets, VRFCert, VRFVKey, Vkey,
};

/// Block0 representation
#[derive(Clone)]
pub struct Block0 {
    /// Block containing transactions
    pub block: Block,
    /// Consensus settings
    pub settings: Settings,
}

impl Default for Block0 {
    fn default() -> Self {
        Self {
            block: BlockBuilder::next_block(None, &[]),
            settings: Settings::default(),
        }
    }
}

/// Block builder responsible for building blocks
pub struct BlockBuilder;

impl BlockBuilder {
    /// Mint new block based on previous block and transactions
    ///
    /// # Panics
    ///
    /// On integer conversion
    #[must_use]
    pub fn next_block(prev: Option<&Block>, transactions: &[Transaction]) -> Block {
        let header_body = Self::block_header(
            prev.map_or(1, |b| b.header().header_body().block_number()),
            prev.map(|b| b.header().header_body().block_body_hash()),
        );

        let header = Header::new(&header_body, &Self::random_kes_signature());

        let bodies = transactions.iter().map(Transaction::body).fold(
            TransactionBodies::new(),
            |mut acc, x| {
                acc.add(&x);
                acc
            },
        );

        let metadata = transactions
            .iter()
            .filter_map(|x| x.auxiliary_data().map(|x| x.metadata()))
            .enumerate()
            .fold(AuxiliaryDataSet::new(), |mut acc, x| {
                let mut auxiliary_data = AuxiliaryData::new();
                if let Some(metadata) = &x.1 {
                    auxiliary_data.set_metadata(metadata);
                }
                acc.insert(u32::try_from(x.0).unwrap(), &auxiliary_data);
                acc
            });

        Block::new(
            &header,
            &bodies,
            &TransactionWitnessSets::new(),
            &metadata,
            vec![],
        )
    }

    fn random_kes_signature() -> KESSignature {
        KESSignature::from_bytes(Self::generate_random_bytes_of_len(KESSignature::BYTE_COUNT))
            .unwrap()
    }

    fn generate_random_bytes_of_len(len: usize) -> Vec<u8> {
        (0..len).map(|_| rand::random::<u8>()).collect()
    }

    /// Builds new block header based on block number and previous block hash
    ///
    /// # Panics
    ///
    /// On random bytes generation issue
    #[must_use]
    pub fn block_header(block_number: u32, prev_hash: Option<BlockHash>) -> HeaderBody {
        let issuer_vkey = PrivateKey::generate_ed25519extended().unwrap().to_public();

        HeaderBody::new_headerbody(
            block_number,
            &BigNum::from(block_number),
            prev_hash,
            &Vkey::new(&issuer_vkey),
            &VRFVKey::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),
            &VRFCert::new(
                Self::generate_random_bytes_of_len(32),
                Self::generate_random_bytes_of_len(VRFCert::PROOF_LEN),
            )
            .unwrap(),
            0,
            &BlockHash::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),
            &OperationalCert::new(
                &KESVKey::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),
                0,
                0,
                &Ed25519Signature::from_bytes(Self::generate_random_bytes_of_len(64)).unwrap(),
            ),
            &ProtocolVersion::new(0, 1),
        )
    }
}
