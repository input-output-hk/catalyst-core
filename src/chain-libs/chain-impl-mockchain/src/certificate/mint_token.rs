use crate::{
    account::Identifier,
    certificate::CertificateSlice,
    tokens::{minting_policy::MintingPolicy, name::TokenName},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    value::Value,
};

use chain_core::mempack::{ReadBuf, ReadError, Readable};
use typed_bytes::ByteBuilder;

use std::marker::PhantomData;

#[derive(Debug, Clone)]
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

impl Readable for MintToken {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        let name = TokenName::read(buf)?;
        let policy = MintingPolicy::read(buf)?;
        let to = Identifier::read(buf)?;
        let value = Value::read(buf)?;

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
}
