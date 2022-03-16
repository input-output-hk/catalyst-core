use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
#[cfg(feature = "evm")]
use chain_evm::Address;
use typed_bytes::{ByteArray, ByteBuilder};

#[cfg(feature = "evm")]
use crate::transaction::UnspecifiedAccountIdentifier;
use crate::transaction::{
    Payload, PayloadAuthData, PayloadData, PayloadSlice, SingleAccountBindingSignature,
};

use super::CertificateSlice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMapping {
    #[cfg(feature = "evm")]
    pub account_id: UnspecifiedAccountIdentifier,
    #[cfg(feature = "evm")]
    pub evm_address: Address,
}

impl EvmMapping {
    #[cfg(feature = "evm")]
    pub fn new(evm_address: Address, account_id: UnspecifiedAccountIdentifier) -> Self {
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
    pub fn account_id(&self) -> &UnspecifiedAccountIdentifier {
        &self.account_id
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        #[cfg(feature = "evm")]
        {
            bb.bytes(self.account_id().as_ref())
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

impl property::Serialize for EvmMapping {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, _writer: W) -> Result<(), Self::Error> {
        #[cfg(feature = "evm")]
        {
            let mut codec = chain_core::packer::Codec::new(_writer);
            codec.put_bytes(self.account_id.as_ref())?;
            codec.put_bytes(self.evm_address.as_bytes())?;
        }
        Ok(())
    }
}

impl property::Deserialize for EvmMapping {
    type Error = std::io::Error;
    fn deserialize<R: std::io::BufRead>(_reader: R) -> Result<Self, Self::Error> {
        #[cfg(feature = "evm")]
        {
            let mut codec = chain_core::packer::Codec::new(_reader);
            let buf: [u8; crate::transaction::INPUT_PTR_SIZE] = codec
                .get_bytes(crate::transaction::INPUT_PTR_SIZE)?
                .try_into()
                .unwrap();
            let evm_address = codec.get_bytes(Address::len_bytes())?;

            Ok(Self {
                account_id: buf.into(),
                evm_address: Address::from_slice(evm_address.as_slice()),
            })
        }
        #[cfg(not(feature = "evm"))]
        Err(Self::Error::new(
            std::io::ErrorKind::Unsupported,
            "evm transactions are not supported in this build",
        ))
    }
}

impl Readable for EvmMapping {
    fn read(_buf: &mut ReadBuf) -> Result<Self, ReadError> {
        #[cfg(feature = "evm")]
        {
            Ok(Self {
                account_id: <[u8; crate::transaction::INPUT_PTR_SIZE]>::read(_buf)?.into(),
                evm_address: Address::from_slice(_buf.get_slice(Address::len_bytes())?),
            })
        }
        #[cfg(not(feature = "evm"))]
        Err(ReadError::InvalidData(
            "evm transactions are not supported in this build".into(),
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
                account_id: [u8::arbitrary(g); crate::transaction::INPUT_PTR_SIZE].into(),
                evm_address: [u8::arbitrary(g); Address::len_bytes()].into(),
            }
        }
    }

    quickcheck! {
        fn evm_transaction_serialization_bijection(b: EvmMapping) -> bool {
            let bytes = b.serialize_in(ByteBuilder::new()).finalize_as_vec();
            let decoded = EvmMapping::read(&mut chain_core::mempack::ReadBuf::from(&bytes)).unwrap();
            decoded == b
        }
    }
}
