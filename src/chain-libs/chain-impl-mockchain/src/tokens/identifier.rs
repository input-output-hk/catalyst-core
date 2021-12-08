use crate::tokens::{
    name::{TokenName, TokenNameTooLong},
    policy_hash::PolicyHash,
};

use std::{convert::TryFrom, fmt, str::FromStr};

use chain_core::mempack::{ReadBuf, ReadError, Readable};
use thiserror::Error;
use typed_bytes::ByteBuilder;

/// The unique identifier of a token.
///
/// It is represented either as two hex strings separated by a dot or just a hex string when the
/// name is empty.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TokenIdentifier {
    pub policy_hash: PolicyHash,
    pub token_name: TokenName,
}

/// Error during parsing the identifier string.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("got an empty str")]
    EmptyStr,

    #[error(transparent)]
    Hex(#[from] hex::FromHexError),

    #[error(transparent)]
    PolicyHash(#[from] ReadError),

    #[error("expected a token name after the `.`")]
    ExpectedTokenName,

    #[error(transparent)]
    TokenName(#[from] TokenNameTooLong),

    #[error("unexpected data after the token name")]
    UnexpectedData,
}

impl TokenIdentifier {
    pub fn bytes(&self) -> Vec<u8> {
        let bb: ByteBuilder<Self> = ByteBuilder::new();
        let token_name = self.token_name.as_ref();
        bb.bytes(self.policy_hash.as_ref())
            .u8(token_name.len() as u8)
            .bytes(token_name)
            .finalize_as_vec()
    }
}

impl Readable for TokenIdentifier {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        let policy_hash = PolicyHash::read(buf)?;
        let token_name = TokenName::read(buf)?;
        Ok(Self {
            policy_hash,
            token_name,
        })
    }
}

impl fmt::Display for TokenIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.policy_hash.as_ref()))?;
        let token_name = self.token_name.as_ref();
        if !token_name.is_empty() {
            write!(f, ".{}", hex::encode(token_name))?;
        }
        Ok(())
    }
}

impl FromStr for TokenIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        let policy_hash = {
            let hex = parts.next().ok_or(ParseError::EmptyStr)?;
            let bytes = hex::decode(hex)?;
            PolicyHash::try_from(bytes.as_ref())?
        };

        let token_name = {
            let bytes = if let Some(hex) = parts.next() {
                hex::decode(hex)?
            } else {
                Vec::new()
            };
            TokenName::try_from(bytes)?
        };

        if parts.next().is_some() {
            return Err(ParseError::UnexpectedData);
        }

        Ok(TokenIdentifier {
            policy_hash,
            token_name,
        })
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for TokenIdentifier {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let policy_hash = Arbitrary::arbitrary(g);
            let token_name = Arbitrary::arbitrary(g);
            Self {
                policy_hash,
                token_name,
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    fn token_identifier_display_sanity(id: TokenIdentifier) {
        let s = id.to_string();
        let id_: TokenIdentifier = s.parse().unwrap();
        assert_eq!(id, id_);
    }

    #[quickcheck_macros::quickcheck]
    fn token_identifier_serialization_bijection(id: TokenIdentifier) -> TestResult {
        let id_got = id.bytes();
        let mut buf = ReadBuf::from(id_got.as_ref());
        let result = TokenIdentifier::read(&mut buf);
        let left = Ok(id);
        assert_eq!(left, result);
        assert_eq!(buf.get_slice_end(), &[]);
        TestResult::from_bool(left == result)
    }
}
