#[cfg(feature = "evm")]
use crate::account::Identifier;
use crate::transaction::{
    Payload, PayloadAuthData, PayloadData, PayloadSlice, SingleAccountBindingSignature,
};
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize, WriteError},
};
#[cfg(feature = "evm")]
use chain_evm::{
    crypto::{
        secp256k1::{Error, Message, RecoverableSignature, RecoveryId},
        sha3::{Digest, Keccak256},
    },
    Address,
};
use typed_bytes::{ByteArray, ByteBuilder};

use super::CertificateSlice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMapping {
    #[cfg(feature = "evm")]
    pub account_id: Identifier,
    #[cfg(feature = "evm")]
    pub evm_address: Address,
}

impl EvmMapping {
    #[cfg(feature = "evm")]
    fn message_for_hash(&self) -> String {
        let msg = format!("{}{}", self.account_id, self.evm_address);
        format!("\x19Ethereum Signed Message:\n{}{:?}", msg.len(), msg)
    }

    #[cfg(feature = "evm")]
    pub fn sign(self, secret: &chain_evm::util::Secret) -> Result<EvmMappingSigned, Error> {
        let msg = self.message_for_hash();
        let (recid, signature_data) =
            chain_evm::util::sign_data(msg.as_bytes(), secret)?.serialize_compact();
        Ok(EvmMappingSigned {
            evm_mapping: self,
            signature_data,
            recid: (recid.to_i32() % 2) as u8,
        })
    }

    #[cfg(feature = "evm")]
    pub fn new(evm_address: Address, account_id: Identifier) -> Self {
        Self {
            account_id,
            evm_address,
        }
    }

    #[cfg(feature = "evm")]
    pub fn evm_address(&self) -> &Address {
        &self.evm_address
    }

    #[cfg(feature = "evm")]
    pub fn account_id(&self) -> &Identifier {
        &self.account_id
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        #[cfg(feature = "evm")]
        {
            bb.bytes(self.account_id.as_ref().as_ref())
                .bytes(self.evm_address.as_bytes())
        }
        #[cfg(not(feature = "evm"))]
        bb
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for EvmMapping {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = SingleAccountBindingSignature;

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        let bb = ByteBuilder::<Self>::new()
            .bytes(auth.as_ref())
            .finalize_as_vec();
        PayloadAuthData(bb.into(), std::marker::PhantomData)
    }

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl Serialize for EvmMapping {
    fn serialized_size(&self) -> usize {
        #[allow(unused_mut)]
        let mut res = 0;
        #[cfg(feature = "evm")]
        {
            res += self.account_id.serialized_size() + self.evm_address.0.serialized_size();
        }
        res
    }

    fn serialize<W: std::io::Write>(&self, _codec: &mut Codec<W>) -> Result<(), WriteError> {
        #[cfg(feature = "evm")]
        {
            self.account_id.serialize(_codec)?;
            _codec.put_bytes(self.evm_address.as_bytes())?;
        }
        Ok(())
    }
}

impl DeserializeFromSlice for EvmMapping {
    fn deserialize_from_slice(_codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        #[cfg(feature = "evm")]
        {
            let account_id = Identifier::deserialize_from_slice(_codec)?;
            let evm_address = _codec.get_bytes(Address::len_bytes())?;

            Ok(Self {
                account_id,
                evm_address: Address::from_slice(evm_address.as_slice()),
            })
        }
        #[cfg(not(feature = "evm"))]
        Err(ReadError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "evm transactions are not supported in this build",
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMappingSigned {
    pub evm_mapping: EvmMapping,
    pub signature_data: [u8; 64],
    pub recid: u8,
}

impl EvmMappingSigned {
    #[cfg(feature = "evm")]
    pub fn recover(&self) -> Result<Address, Error> {
        let msg_for_hash = self.evm_mapping.message_for_hash();
        let recid = RecoveryId::from_i32(self.recid as i32)?;
        let signature = RecoverableSignature::from_compact(&self.signature_data, recid)?;
        let msg = Message::from_slice(msg_for_hash.as_bytes())?;
        let pubkey = signature.recover(&msg)?;
        let pubkey_bytes = pubkey.serialize_uncompressed();
        Ok(Address::from_slice(
            &Keccak256::digest(&pubkey_bytes[1..]).as_slice()[12..],
        ))
    }
}

#[cfg(all(any(test, feature = "property-test-api"), feature = "evm"))]
mod test {
    use super::*;
    use quickcheck::Arbitrary;

    impl Arbitrary for EvmMapping {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            Self {
                account_id: Arbitrary::arbitrary(g),
                evm_address: [u8::arbitrary(g); Address::len_bytes()].into(),
            }
        }
    }

    quickcheck! {
        fn evm_transaction_serialization_bijection(b: EvmMapping) -> bool {
            let bytes = b.serialize_in(ByteBuilder::new()).finalize_as_vec();
            let decoded = EvmMapping::deserialize_from_slice(&mut Codec::new(bytes.as_slice())).unwrap();
            decoded == b
        }
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod prop_impl {
    use proptest::prelude::*;

    #[cfg(feature = "evm")]
    use crate::account::Identifier;
    use crate::certificate::EvmMapping;
    #[cfg(feature = "evm")]
    use chain_evm::Address;
    #[cfg(feature = "evm")]
    use proptest::{arbitrary::StrategyFor, strategy::Map};

    impl Arbitrary for EvmMapping {
        type Parameters = ();

        #[cfg(not(feature = "evm"))]
        type Strategy = Just<Self>;
        #[cfg(not(feature = "evm"))]
        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            Just(Self {})
        }

        #[cfg(feature = "evm")]
        type Strategy =
            Map<StrategyFor<(Identifier, [u8; 20])>, fn((Identifier, [u8; 20])) -> Self>;

        #[cfg(feature = "evm")]
        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(Identifier, [u8; 20])>().prop_map(|(account_id, evm_address)| Self {
                account_id,
                evm_address: Address::from_slice(&evm_address),
            })
        }
    }
}
