use crate::MainnetWallet;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::chain_crypto::Blake2b256;
use cardano_serialization_lib::crypto::{Ed25519Signature, PublicKey};
use cardano_serialization_lib::metadata::{
    GeneralTransactionMetadata, MetadataMap, TransactionMetadatum, TransactionMetadatumLabel,
};
use cardano_serialization_lib::utils::{BigNum, Int};
use snapshot_lib::registration::Delegations;

pub struct RegistrationBuilder<'a> {
    wallet: &'a MainnetWallet,
    delegations: Option<Delegations>,
    slot_no: u64,
}

impl<'a> RegistrationBuilder<'a> {
    pub fn new(wallet: &'a MainnetWallet) -> Self {
        Self {
            wallet,
            delegations: None,
            slot_no: 0,
        }
    }

    pub fn to(mut self, delegations: Delegations) -> Self {
        self.delegations = Some(delegations);
        self
    }

    pub fn on(mut self, slot_no: u64) -> Self {
        self.slot_no = slot_no;
        self
    }

    pub fn build(self) -> GeneralTransactionMetadata {
        let mut meta_map: MetadataMap = MetadataMap::new();

        match self.delegations.expect("no registration target defined") {
            Delegations::New(_delegations) => (),
            Delegations::Legacy(legacy) => {
                meta_map.insert(
                    &TransactionMetadatum::new_int(&Int::new_i32(1)),
                    &TransactionMetadatum::new_bytes(hex::decode(legacy.to_hex()).unwrap())
                        .unwrap(),
                );
            }
        };

        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(2)),
            &TransactionMetadatum::new_bytes(self.wallet.stake_public_key().as_bytes()).unwrap(),
        );
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(3)),
            &TransactionMetadatum::new_bytes(self.wallet.reward_address().to_address().to_bytes())
                .unwrap(),
        );
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(4)),
            &TransactionMetadatum::new_int(&Int::new(&BigNum::from(self.slot_no))),
        );

        let mut metadata = GeneralTransactionMetadata::new();
        metadata.insert(
            &BigNum::from(61284u32),
            &TransactionMetadatum::new_map(&meta_map),
        );

        let meta_bytes = metadata.to_bytes();
        let meta_bytes_hash = Blake2b256::new(&meta_bytes);
        let signature = self.wallet.stake_key.sign(meta_bytes_hash.as_hash_bytes());

        let mut meta_sign_map: MetadataMap = MetadataMap::new();

        meta_sign_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(1)),
            &TransactionMetadatum::new_bytes(signature.to_bytes()).unwrap(),
        );

        metadata.insert(
            &BigNum::from(61285u32),
            &TransactionMetadatum::new_map(&meta_sign_map),
        );

        metadata
    }
}

pub trait GeneralTransactionMetadataInfo {
    fn delegations(&self) -> Vec<u8>;
    fn stake_public_key(&self) -> PublicKey;
    fn reward_address(&self) -> Address;
    fn signature(&self) -> Ed25519Signature;
    fn registration_blake_256_hash(&self) -> Blake2b256;
}

impl GeneralTransactionMetadataInfo for GeneralTransactionMetadata {
    fn delegations(&self) -> Vec<u8> {
        let metadata = self
            .get(&TransactionMetadatumLabel::from(61284u32))
            .unwrap();
        let metadata_map = metadata.as_map().unwrap();
        let metadata = metadata_map
            .get(&TransactionMetadatum::new_int(&Int::new_i32(1)))
            .unwrap();
        metadata.as_bytes().unwrap()
    }

    fn stake_public_key(&self) -> PublicKey {
        let metadata = self
            .get(&TransactionMetadatumLabel::from(61284u32))
            .unwrap();
        let metadata_map = metadata.as_map().unwrap();
        PublicKey::from_bytes(
            &metadata_map
                .get(&TransactionMetadatum::new_int(&Int::new_i32(2)))
                .unwrap()
                .as_bytes()
                .unwrap(),
        )
        .unwrap()
    }

    fn reward_address(&self) -> Address {
        let metadata = self
            .get(&TransactionMetadatumLabel::from(61284u32))
            .unwrap();
        let metadata_map = metadata.as_map().unwrap();
        Address::from_bytes(
            metadata_map
                .get(&TransactionMetadatum::new_int(&Int::new_i32(3)))
                .unwrap()
                .as_bytes()
                .unwrap(),
        )
        .unwrap()
    }

    fn signature(&self) -> Ed25519Signature {
        let signature_metadata = self
            .get(&TransactionMetadatumLabel::from(61285u32))
            .unwrap();
        let signature_metadata_map = signature_metadata.as_map().unwrap();
        Ed25519Signature::from_bytes(
            signature_metadata_map
                .get(&TransactionMetadatum::new_int(&Int::new_i32(1)))
                .unwrap()
                .as_bytes()
                .unwrap(),
        )
        .unwrap()
    }

    fn registration_blake_256_hash(&self) -> Blake2b256 {
        let metadata = self
            .get(&TransactionMetadatumLabel::from(61284u32))
            .unwrap();
        let meta_bytes = metadata.to_bytes();
        Blake2b256::new(&meta_bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jormungandr_lib::crypto::account::Identifier;

    #[test]
    pub fn cip15_registration() {
        let wallet = MainnetWallet::new(1);
        let metadata = RegistrationBuilder::new(&wallet)
            .to(Delegations::Legacy(wallet.catalyst_public_key()))
            .build();

        assert_eq!(
            Identifier::from_hex(&hex::encode(metadata.delegations())).unwrap(),
            wallet.catalyst_public_key()
        );
        assert_eq!(metadata.stake_public_key(), wallet.stake_public_key());
        assert_eq!(
            metadata.reward_address(),
            wallet.reward_address().to_address()
        );
        assert!(wallet.stake_public_key().verify(
            metadata.registration_blake_256_hash().as_hash_bytes(),
            &metadata.signature()
        ));
    }
}
