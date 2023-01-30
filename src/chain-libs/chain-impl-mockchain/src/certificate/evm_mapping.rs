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
pub struct EvmMapping {}

impl EvmMapping {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
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
        res
    }

    fn serialize<W: std::io::Write>(&self, _codec: &mut Codec<W>) -> Result<(), WriteError> {
        Ok(())
    }
}

impl DeserializeFromSlice for EvmMapping {
    fn deserialize_from_slice(_codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
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

#[cfg(any(test, feature = "property-test-api"))]
mod prop_impl {
    use proptest::prelude::*;

    use crate::certificate::EvmMapping;

    impl Arbitrary for EvmMapping {
        type Parameters = ();

        type Strategy = Just<Self>;
        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            Just(Self {})
        }
    }
}
