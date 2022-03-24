use crate::{
    account::Identifier,
    certificate::CertificateSlice,
    tokens::{minting_policy::MintingPolicy, name::TokenName},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    value::Value,
};
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};
use typed_bytes::{ByteArray, ByteBuilder};

use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintToken {
    pub name: TokenName,
    pub policy: MintingPolicy,
    pub to: Identifier,
    pub value: Value,
}

impl MintToken {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        let name = self.name.as_ref();
        bb.u8(name.len() as u8)
            .bytes(name)
            .bytes(&self.policy.bytes())
            .bytes(self.to.as_ref().as_ref())
            .bytes(&self.value.bytes())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl Payload for MintToken {
    const HAS_DATA: bool = true;

    const HAS_AUTH: bool = false;

    type Auth = ();

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }

    fn payload_auth_data(_: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(Box::new([]), PhantomData)
    }

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

impl Serialize for MintToken {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        self.name.serialize(codec)?;
        self.policy.serialize(codec)?;
        self.to.serialize(codec)?;
        self.value.serialize(codec)
    }
}

impl DeserializeFromSlice for MintToken {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let name = TokenName::deserialize(codec)?;
        let policy = MintingPolicy::deserialize(codec)?;
        let to = Identifier::deserialize_from_slice(codec)?;
        let value = Value::deserialize(codec)?;

        Ok(Self {
            name,
            policy,
            to,
            value,
        })
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[cfg(test)]
    use crate::testing::serialization::serialization_bijection;
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for MintToken {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let name = Arbitrary::arbitrary(g);
            let policy = MintingPolicy::arbitrary(g);
            let to = Arbitrary::arbitrary(g);
            let value = Arbitrary::arbitrary(g);
            Self {
                name,
                policy,
                to,
                value,
            }
        }
    }

    quickcheck! {
        fn minttoken_serialization_bijection(b: MintToken) -> TestResult {
            serialization_bijection(b)
        }
    }
}
