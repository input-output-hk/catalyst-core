use crate::vote::Choice;
use chain_core::mempack::{ReadBuf, ReadError};
use std::convert::{TryFrom, TryInto as _};
use thiserror::Error;
use typed_bytes::ByteBuilder;

/// the `PayloadType` to use for a vote plan
///
/// this defines how the vote must be published on chain.
/// Be careful because the default is set to `Public`.
///
/// ```
/// use chain_impl_mockchain::vote::PayloadType;
/// assert_eq!(PayloadType::Public, PayloadType::default());
/// ```
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum PayloadType {
    Public = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Payload {
    Public { choice: Choice },
}

#[derive(Debug, Error)]
pub enum TryFromIntError {
    #[error("Found a `0` PayloadType. This is unexpected and known to be an error to read a 0.")]
    Zero,
    #[error("invalid value for a PayloadType")]
    InvalidValue { value: u8 },
}

impl Payload {
    pub fn public(choice: Choice) -> Self {
        Self::Public { choice }
    }

    pub fn payload_type(&self) -> PayloadType {
        match self {
            Self::Public { .. } => PayloadType::Public,
        }
    }

    pub(crate) fn serialize_in<T>(&self, bb: ByteBuilder<T>) -> ByteBuilder<T> {
        let payload_type = self.payload_type();

        match self {
            Self::Public { choice } => bb.u8(payload_type as u8).u8(choice.as_byte()),
        }
    }

    pub(crate) fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let t = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        match t {
            PayloadType::Public => buf.get_u8().map(Choice::new).map(Self::public),
        }
    }
}

impl TryFrom<u8> for PayloadType {
    type Error = TryFromIntError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Err(TryFromIntError::Zero),
            1 => Ok(Self::Public),
            _ => Err(TryFromIntError::InvalidValue { value }),
        }
    }
}

impl Default for PayloadType {
    fn default() -> Self {
        PayloadType::Public
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for PayloadType {
        fn arbitrary<G: Gen>(_g: &mut G) -> Self {
            Self::Public
        }
    }

    impl Arbitrary for Payload {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match PayloadType::arbitrary(g) {
                PayloadType::Public => Payload::public(Choice::arbitrary(g)),
            }
        }
    }
}
