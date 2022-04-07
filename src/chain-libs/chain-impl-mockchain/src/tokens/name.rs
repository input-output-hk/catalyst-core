use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};
use thiserror::Error;
use typed_bytes::ByteBuilder;

pub const TOKEN_NAME_MAX_SIZE: usize = 32;

/// A sequence of bytes serving as a token name. Tokens that share the same name but have different
/// voting policies hashes are different tokens. A name can be empty. The maximum length of a token
/// name is 32 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct TokenName(Vec<u8>);

#[derive(Debug, Error)]
#[error("Token name can be no more that {} bytes long; got {} bytes", TOKEN_NAME_MAX_SIZE, .actual)]
pub struct TokenNameTooLong {
    actual: usize,
}

impl AsRef<[u8]> for TokenName {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TokenName {
    pub fn bytes(&self) -> Vec<u8> {
        let bb: ByteBuilder<Self> = ByteBuilder::new();
        bb.u8(self.0.len() as u8)
            .bytes(self.0.as_ref())
            .finalize_as_vec()
    }
}

impl TryFrom<Vec<u8>> for TokenName {
    type Error = TokenNameTooLong;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() > TOKEN_NAME_MAX_SIZE {
            return Err(TokenNameTooLong {
                actual: value.len(),
            });
        }
        Ok(Self(value))
    }
}

impl Serialize for TokenName {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_u8(self.0.len() as u8)?;
        codec.put_bytes(self.0.as_slice())
    }
}

impl Deserialize for TokenName {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let name_length = codec.get_u8()? as usize;
        if name_length > TOKEN_NAME_MAX_SIZE {
            return Err(ReadError::SizeTooBig(TOKEN_NAME_MAX_SIZE, name_length));
        }
        let bytes = codec.get_bytes(name_length)?;
        Ok(Self(bytes))
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for TokenName {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let len = usize::arbitrary(g) % (TOKEN_NAME_MAX_SIZE + 1);
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                bytes.push(Arbitrary::arbitrary(g));
            }
            Self(bytes)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn token_name_serialization_bijection(token_name: TokenName) -> TestResult {
        let token_name_got = token_name.bytes();
        let mut codec = Codec::new(token_name_got.as_slice());
        let result = TokenName::deserialize(&mut codec).unwrap();
        TestResult::from_bool(token_name == result)
    }
}
