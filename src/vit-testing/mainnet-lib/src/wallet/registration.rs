use crate::cardano_node::TransactionBuilder;
use crate::CardanoWallet;
use cardano_serialization_lib::{
    Address, BigNum, Ed25519Signature, GeneralTransactionMetadata, Int, JsError,
    MetadataJsonSchema, MetadataList, MetadataMap, PublicKey, TransactionMetadatum,
    TransactionMetadatumLabel, decode_metadatum_to_json_str, encode_json_value_to_metadatum,
};
use cardano_serialization_lib::chain_crypto::Blake2b256;
use cardano_serialization_lib::Transaction;
use serde_json::Map;
use snapshot_lib::registration::Delegations;

lazy_static::lazy_static! {
    /// registration metadata index constant
   pub static ref REGISTRATION_METADATA_IDX: u32 = 61284u32;
    /// registration signature metadata index constant
    pub static ref REGISTRATION_SIGNATURE_METADATA_IDX: u32 = 61285u32;
    /// registration metadata constant
   pub static ref REGISTRATION_METADATA_LABEL: TransactionMetadatumLabel = TransactionMetadatumLabel::from(*REGISTRATION_METADATA_IDX);
    ///registration signature metadata constant
    pub static ref REGISTRATION_METADATA_SIGNATURE_LABEL: TransactionMetadatumLabel = TransactionMetadatumLabel::from(*REGISTRATION_SIGNATURE_METADATA_IDX);
    /// metadatum label 1 constant
   pub static ref METADATUM_1: TransactionMetadatum = TransactionMetadatum::new_int(&Int::new_i32(1));
    /// metadatum label 2 constant
    pub static ref METADATUM_2: TransactionMetadatum = TransactionMetadatum::new_int(&Int::new_i32(2));
    /// metadatum label 3 constant
    pub  static ref METADATUM_3: TransactionMetadatum = TransactionMetadatum::new_int(&Int::new_i32(3));
    /// metadatum label 4 constant
    pub  static ref METADATUM_4: TransactionMetadatum = TransactionMetadatum::new_int(&Int::new_i32(4));
    /// metadatum label 5 constant
    pub  static ref METADATUM_5: TransactionMetadatum = TransactionMetadatum::new_int(&Int::new_i32(5));
}

/// Responsible for building registration transaction metadata
pub struct RegistrationTransactionBuilder<'a> {
    wallet: &'a CardanoWallet,
    delegations: Option<Delegations>,
    slot_no: u64,
}

impl<'a> RegistrationTransactionBuilder<'a> {
    /// Creates registration builder for given wallet
    #[must_use]
    pub fn new(wallet: &'a CardanoWallet) -> Self {
        Self {
            wallet,
            delegations: None,
            slot_no: 0,
        }
    }

    /// Defines registration target (self or delegated)
    #[must_use]
    pub fn to(mut self, delegations: Delegations) -> Self {
        self.delegations = Some(delegations);
        self
    }

    /// Defines slot number for registration transaction. This will be used as nonce
    #[must_use]
    pub fn on(mut self, slot_no: u64) -> Self {
        self.slot_no = slot_no;
        self
    }

    /// Creates transaction metadata
    ///
    /// # Panics
    ///
    /// On metadata size overflow
    #[must_use]
    fn build_metadata(&self) -> GeneralTransactionMetadata {
        let mut meta_map: MetadataMap = MetadataMap::new();

        let delegation_metadata = match self
            .delegations
            .as_ref()
            .expect("no registration target defined")
        {
            Delegations::New(delegations) => {
                let mut metadata_list = MetadataList::new();
                for (delegation, weight) in delegations {
                    let mut inner_metadata_list = MetadataList::new();
                    inner_metadata_list.add(
                        &TransactionMetadatum::new_bytes(hex::decode(delegation.to_hex()).unwrap())
                            .unwrap(),
                    );
                    inner_metadata_list.add(&TransactionMetadatum::new_int(&Int::new(
                        &BigNum::from(*weight),
                    )));
                    metadata_list.add(&TransactionMetadatum::new_list(&inner_metadata_list));
                }
                TransactionMetadatum::new_list(&metadata_list)
            }
            Delegations::Legacy(legacy) => {
                TransactionMetadatum::new_bytes(hex::decode(legacy.to_hex()).unwrap()).unwrap()
            }
        };

        meta_map.insert(&METADATUM_1, &delegation_metadata);

        meta_map.insert(
            &METADATUM_2,
            &TransactionMetadatum::new_bytes(self.wallet.stake_public_key().as_bytes()).unwrap(),
        );
        meta_map.insert(
            &METADATUM_3,
            &TransactionMetadatum::new_bytes(self.wallet.reward_address().to_address().to_bytes())
                .unwrap(),
        );
        meta_map.insert(
            &METADATUM_4,
            &TransactionMetadatum::new_int(&Int::new(&BigNum::from(self.slot_no))),
        );
        meta_map.insert(
            &METADATUM_5,
            &TransactionMetadatum::new_int(&Int::new(&BigNum::zero())),
        );

        let mut metadata = GeneralTransactionMetadata::new();
        metadata.insert(
            &BigNum::from(*REGISTRATION_METADATA_IDX),
            &TransactionMetadatum::new_map(&meta_map),
        );

        let meta_bytes = metadata.to_bytes();
        let meta_bytes_hash = Blake2b256::new(&meta_bytes);
        let signature = self.wallet.stake_key.sign(meta_bytes_hash.as_hash_bytes());

        let mut meta_sign_map: MetadataMap = MetadataMap::new();

        meta_sign_map.insert(
            &METADATUM_1,
            &TransactionMetadatum::new_bytes(signature.to_bytes()).unwrap(),
        );

        metadata.insert(
            &BigNum::from(*REGISTRATION_SIGNATURE_METADATA_IDX),
            &TransactionMetadatum::new_map(&meta_sign_map),
        );
        metadata
    }

    /// Builds transaction instance
    #[must_use]
    pub fn build(self) -> Transaction {
        let metadata = self.build_metadata();
        TransactionBuilder::build_transaction_with_metadata(
            &self.wallet.reward_address().to_address(),
            self.wallet.stake,
            &metadata,
        )
    }
}

/// Metadata conversion error
#[derive(thiserror::Error, Debug)]
pub enum JsonConversionError {
    /// Serialization
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    /// Missing registration label
    #[error("missing registration label in json")]
    MissingRegistrationLabel,
    /// Missing registration signature label
    #[error("missing registration signature label in json")]
    MissingRegistrationSignatureLabel,
    /// Internal error
    #[error(transparent)]
    Internal(#[from] JsError),
    /// Incorrect schema
    #[error("incorrect input json: root is not a map")]
    IncorrectInputJson,
}

/// Extension for `GeneralTransactionMetadata` tailored for Catalyst purposes
pub trait GeneralTransactionMetadataInfo {
    /// Converts metadata to json
    ///
    /// # Errors
    ///
    /// On json conversion error
    fn to_json_string(&self, schema: MetadataJsonSchema) -> Result<String, JsonConversionError>;
    /// Converts json to metadata
    ///
    /// # Errors
    ///
    /// On json conversion error
    fn from_json_string(
        json: &str,
        schema: MetadataJsonSchema,
    ) -> Result<Self, JsonConversionError>
    where
        Self: Sized;

    /// Converts combined jsons to registration and registration metadata
    ///
    /// # Errors
    ///
    /// On json conversion error
    fn from_jsons(
        reg_metadata: serde_json::Value,
        signature_metadata: serde_json::Value,
        schema: MetadataJsonSchema,
    ) -> Result<Self, JsonConversionError>
    where
        Self: Sized;

    /// Get delegations part as bytes
    fn delegations(&self) -> Vec<u8>;
    /// Stake public key
    fn stake_public_key(&self) -> PublicKey;
    /// Reward address
    fn reward_address(&self) -> Address;
    /// metadata signature
    fn signature(&self) -> Ed25519Signature;
    /// registration metadata hash
    fn registration_blake_256_hash(&self) -> Blake2b256;
    /// nonce
    fn nonce(&self) -> i32;
}

impl GeneralTransactionMetadataInfo for GeneralTransactionMetadata {
    fn to_json_string(&self, schema: MetadataJsonSchema) -> Result<String, JsonConversionError> {
        let reg_metadata = self
            .get(&REGISTRATION_METADATA_LABEL)
            .ok_or(JsonConversionError::MissingRegistrationLabel)?;
        let signature_metadata = self
            .get(&REGISTRATION_METADATA_SIGNATURE_LABEL)
            .ok_or(JsonConversionError::MissingRegistrationSignatureLabel)?;
        let registration_json = decode_metadatum_to_json_str(&reg_metadata, schema)?;
        let registration_sig_json = decode_metadatum_to_json_str(&signature_metadata, schema)?;

        let mut map = Map::new();
        map.insert(
            REGISTRATION_METADATA_IDX.to_string(),
            serde_json::from_str(&registration_json)?,
        );
        map.insert(
            REGISTRATION_SIGNATURE_METADATA_IDX.to_string(),
            serde_json::from_str(&registration_sig_json)?,
        );
        serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(Into::into)
    }

    fn from_json_string(
        json: &str,
        schema: MetadataJsonSchema,
    ) -> Result<Self, JsonConversionError> {
        let json: serde_json::Value = serde_json::from_str(json)?;

        let map = json
            .as_object()
            .ok_or(JsonConversionError::IncorrectInputJson)?;
        let registration_json = encode_json_value_to_metadatum(
            map.get(&REGISTRATION_METADATA_IDX.to_string())
                .ok_or(JsonConversionError::MissingRegistrationLabel)?
                .clone(),
            schema,
        )?;
        let registration_sig_json = encode_json_value_to_metadatum(
            map.get(&REGISTRATION_SIGNATURE_METADATA_IDX.to_string())
                .ok_or(JsonConversionError::MissingRegistrationSignatureLabel)?
                .clone(),
            schema,
        )?;

        let mut root_metadata = GeneralTransactionMetadata::new();
        root_metadata.insert(&REGISTRATION_METADATA_LABEL, &registration_json);
        root_metadata.insert(
            &REGISTRATION_METADATA_SIGNATURE_LABEL,
            &registration_sig_json,
        );

        Ok(root_metadata)
    }

    fn from_jsons(
        reg_metadata: serde_json::Value,
        signature_metadata: serde_json::Value,
        schema: MetadataJsonSchema,
    ) -> Result<Self, JsonConversionError>
    where
        Self: Sized,
    {
        let registration_json = encode_json_value_to_metadatum(reg_metadata, schema)?;
        let registration_sig_json = encode_json_value_to_metadatum(signature_metadata, schema)?;

        let mut root_metadata = GeneralTransactionMetadata::new();
        root_metadata.insert(&REGISTRATION_METADATA_LABEL, &registration_json);
        root_metadata.insert(
            &REGISTRATION_METADATA_SIGNATURE_LABEL,
            &registration_sig_json,
        );

        Ok(root_metadata)
    }

    fn delegations(&self) -> Vec<u8> {
        let metadata = self.get(&REGISTRATION_METADATA_LABEL).unwrap();
        let metadata_map = metadata.as_map().unwrap();
        let metadata = metadata_map.get(&METADATUM_1).unwrap();
        metadata.as_bytes().unwrap()
    }

    fn stake_public_key(&self) -> PublicKey {
        let metadata = self.get(&REGISTRATION_METADATA_LABEL).unwrap();
        let metadata_map = metadata.as_map().unwrap();
        PublicKey::from_bytes(&metadata_map.get(&METADATUM_2).unwrap().as_bytes().unwrap()).unwrap()
    }

    fn reward_address(&self) -> Address {
        let metadata = self.get(&REGISTRATION_METADATA_LABEL).unwrap();
        let metadata_map = metadata.as_map().unwrap();
        Address::from_bytes(metadata_map.get(&METADATUM_3).unwrap().as_bytes().unwrap()).unwrap()
    }

    fn signature(&self) -> Ed25519Signature {
        let signature_metadata = self.get(&REGISTRATION_METADATA_SIGNATURE_LABEL).unwrap();
        let signature_metadata_map = signature_metadata.as_map().unwrap();
        Ed25519Signature::from_bytes(
            signature_metadata_map
                .get(&METADATUM_1)
                .unwrap()
                .as_bytes()
                .unwrap(),
        )
        .unwrap()
    }

    fn registration_blake_256_hash(&self) -> Blake2b256 {
        let reg_metadata = self.get(&REGISTRATION_METADATA_LABEL).unwrap();
        let mut metadata = GeneralTransactionMetadata::new();
        metadata.insert(&BigNum::from(*REGISTRATION_METADATA_IDX), &reg_metadata);

        let meta_bytes = metadata.to_bytes();
        Blake2b256::new(&meta_bytes)
    }

    fn nonce(&self) -> i32 {
        let metadata = self.get(&REGISTRATION_METADATA_LABEL).unwrap();
        let metadata_map = metadata.as_map().unwrap();
        metadata_map
            .get(&METADATUM_4)
            .unwrap()
            .as_int()
            .unwrap()
            .as_i32_or_fail()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jormungandr_lib::crypto::account::Identifier;

    #[test]
    pub fn cip15_registration() {
        let wallet = CardanoWallet::new(1);
        let transaction = RegistrationTransactionBuilder::new(&wallet)
            .to(Delegations::Legacy(wallet.catalyst_public_key()))
            .build();

        let metadata = transaction.auxiliary_data().unwrap().metadata().unwrap();

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

    #[test]
    /// this test is more like documentation since it took me a while to understand how to convert
    /// metadata string shown in for example explorer to this format
    pub fn metadata_serialization_bijection() {
        let metadata_string = r#"{"1":"0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663","2":"0x38ee57ed01f04e7bf553f85e6115faa3f0fc94e2a7bf471c939015d716d5dbe7","3":"0xe0ffc025718baab0ee4cf247706e419ae25d3f1be04006d2d8214de3ed","4":11614075,"5":0}"#;
        let metadata_sig_string = r#"{"1":"0x1249f6c152e555af356ae17746457669ae40bcee11ad0671a9a7e9e55441da8a12b152f67c88c15c46f938fd5a0a360dff50e12beeb48556e54ab1e3ea684108"}"#;

        let reg_metadata = encode_json_value_to_metadatum(
            serde_json::from_str(metadata_string).unwrap(),
            MetadataJsonSchema::BasicConversions,
        )
        .unwrap();

        let reg_sig_metadata = encode_json_value_to_metadatum(
            serde_json::from_str(metadata_sig_string).unwrap(),
            MetadataJsonSchema::BasicConversions,
        )
        .unwrap();

        assert_eq!(
            metadata_string,
            decode_metadatum_to_json_str(&reg_metadata, MetadataJsonSchema::BasicConversions)
                .unwrap()
        );
        assert_eq!(
            metadata_sig_string,
            decode_metadatum_to_json_str(&reg_sig_metadata, MetadataJsonSchema::BasicConversions)
                .unwrap()
        );
    }

    #[test]
    pub fn root_metadata_serialization_bijection_direct() {
        let metadata_string = r#"{
            "61284": {
                "1":"0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663",
                "2":"0x38ee57ed01f04e7bf553f85e6115faa3f0fc94e2a7bf471c939015d716d5dbe7",
                "3":"0xe0ffc025718baab0ee4cf247706e419ae25d3f1be04006d2d8214de3ed",
                "4":11614075,
                "5":0
            },
            "61285": {
                "1":"0x1249f6c152e555af356ae17746457669ae40bcee11ad0671a9a7e9e55441da8a12b152f67c88c15c46f938fd5a0a360dff50e12beeb48556e54ab1e3ea684108"
            }
         }"#;

        let root_metadata = GeneralTransactionMetadata::from_json_string(
            metadata_string,
            MetadataJsonSchema::BasicConversions,
        )
        .unwrap();
        let left: serde_json::Value = serde_json::from_str(metadata_string).unwrap();
        let right: serde_json::Value = serde_json::from_str(
            &root_metadata
                .to_json_string(MetadataJsonSchema::BasicConversions)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(left, right);
    }

    #[test]
    pub fn root_metadata_serialization_bijection_delegated() {
        let metadata_string = r#"{
            "61284": {
                "1":[["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 1], ["0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee", 3]],
                "2":"0x38ee57ed01f04e7bf553f85e6115faa3f0fc94e2a7bf471c939015d716d5dbe7",
                "3":"0xe0ffc025718baab0ee4cf247706e419ae25d3f1be04006d2d8214de3ed",
                "4":11614075,
                "5":0
            },
            "61285": {
                "1":"0x1249f6c152e555af356ae17746457669ae40bcee11ad0671a9a7e9e55441da8a12b152f67c88c15c46f938fd5a0a360dff50e12beeb48556e54ab1e3ea684108"
            }
         }"#;

        let root_metadata = GeneralTransactionMetadata::from_json_string(
            metadata_string,
            MetadataJsonSchema::BasicConversions,
        )
        .unwrap();
        let left: serde_json::Value = serde_json::from_str(metadata_string).unwrap();
        let right: serde_json::Value = serde_json::from_str(
            &root_metadata
                .to_json_string(MetadataJsonSchema::BasicConversions)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(left, right);
    }
}
