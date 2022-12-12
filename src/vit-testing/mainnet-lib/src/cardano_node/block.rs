use cardano_serialization_lib::{AuxiliaryDataSet, Block, Header, HeaderBody, OperationalCert, ProtocolVersion, Transaction, TransactionBodies, TransactionWitnessSets};
use cardano_serialization_lib::metadata::AuxiliaryData;
use cardano_serialization_lib::crypto::{BlockHash, Ed25519Signature, KESSignature, KESVKey, PrivateKey, Vkey, VRFCert, VRFVKey};
use cardano_serialization_lib::utils::BigNum;
use crate::Settings;


pub struct Block0{
    pub block: Block,
    pub settings: Settings
}


impl Default for Block0 {
    fn default() -> Self {
        Self {
            block: BlockBuilder::next_block(None,vec![]),
            settings: Default::default()
        }
    }
}

pub struct BlockBuilder;

impl BlockBuilder {

    pub fn next_block(prev: Option<&Block>, transactions: Vec<Transaction>) -> Block {
        let header_body = Self::block_header(
            prev.map(|b| b.header().header_body().block_number()).unwrap_or(1),
            prev.map(|b| b.header().header_body().block_body_hash())
        );

        let header = Header::new(&header_body,&Self::random_kes_signature());

        let bodies = transactions.iter().map(|x| x.body()).fold(TransactionBodies::new(),|mut acc,x| {
                acc.add(&x);
                acc
            });

        let metadata = transactions.iter().filter_map(|x| x.auxiliary_data().map(|x| x.metadata())).enumerate().fold(AuxiliaryDataSet::new(),|mut acc,x| {
            let mut auxiliary_data = AuxiliaryData::new();
            if let Some(metadata) = &x.1 {
                auxiliary_data.set_metadata(metadata);
            }
            acc.insert(x.0 as u32, &auxiliary_data);
            acc
        });

        Block::new(&header,&bodies,&TransactionWitnessSets::new(), &metadata, vec![])
    }

    fn random_kes_signature() -> KESSignature {
        KESSignature::from_bytes(Self::generate_random_bytes_of_len(KESSignature::BYTE_COUNT)).unwrap()
    }

    fn generate_random_bytes_of_len(len: usize) -> Vec<u8> {
        (0..len).map(|_| { rand::random::<u8>() }).collect()
    }

    pub fn block_header(block_number: u32, prev_hash: Option<BlockHash>) -> HeaderBody {
        let issuer_vkey = PrivateKey::generate_ed25519extended().unwrap().to_public();

        HeaderBody::new_headerbody(block_number,
                                   &BigNum::from(block_number),
                                   prev_hash,
                                   &Vkey::new(&issuer_vkey),
                                   &VRFVKey::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),
                                   &VRFCert::new(Self::generate_random_bytes_of_len(32),Self::generate_random_bytes_of_len(VRFCert::PROOF_LEN)).unwrap(),
                                   0,
                                   &BlockHash::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),
                                   &OperationalCert::new(&KESVKey::from_bytes(Self::generate_random_bytes_of_len(32)).unwrap(),0,0,&Ed25519Signature::from_bytes(Self::generate_random_bytes_of_len(64)).unwrap()), &ProtocolVersion::new(0,1))

    }
}
