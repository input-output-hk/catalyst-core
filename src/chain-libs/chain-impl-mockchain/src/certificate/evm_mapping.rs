#[cfg(feature = "evm")]
use crate::account::Identifier;
#[cfg(feature = "evm")]
use crate::evm::Address;
use crate::transaction::{
    Payload, PayloadAuthData, PayloadData, PayloadSlice, SingleAccountBindingSignature,
};
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize, WriteError},
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
