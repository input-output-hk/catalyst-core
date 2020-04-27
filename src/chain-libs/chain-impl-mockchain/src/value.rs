use crate::stake::Stake;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;
use std::convert::TryFrom;
use std::{iter::Sum, ops};
use thiserror::Error;

/// Unspent transaction value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Value(pub u64);

const VALUE_SERIALIZED_SIZE: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SplitValueIn {
    pub parts: Value,
    pub remaining: Value,
}

impl Value {
    pub fn zero() -> Self {
        Value(0)
    }

    pub fn sum<I>(values: I) -> Result<Self, ValueError>
    where
        I: Iterator<Item = Self>,
    {
        values.fold(Ok(Value::zero()), |acc, v| acc? + v)
    }

    #[inline]
    pub fn saturating_add(self, other: Self) -> Self {
        Value(self.0.saturating_add(other.0))
    }

    #[inline]
    pub fn checked_add(self, other: Self) -> Result<Self, ValueError> {
        self.0
            .checked_add(other.0)
            .map(Value)
            .ok_or(ValueError::Overflow)
    }

    #[inline]
    pub fn checked_sub(self, other: Value) -> Result<Value, ValueError> {
        self.0
            .checked_sub(other.0)
            .map(Value)
            .ok_or(ValueError::NegativeAmount)
    }

    pub fn scale(self, n: u32) -> Result<Value, ValueError> {
        self.0
            .checked_mul(n as u64)
            .map(Value)
            .ok_or(ValueError::Overflow)
    }

    /// Divide a value by n equals parts, with a potential remainder
    pub fn split_in(self, n: u32) -> SplitValueIn {
        let n = n as u64;
        SplitValueIn {
            parts: Value(self.0 / n),
            remaining: Value(self.0 % n),
        }
    }

    pub fn bytes(self) -> [u8; VALUE_SERIALIZED_SIZE] {
        self.0.to_be_bytes()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValueError {
    #[error("Value cannot be negative")]
    NegativeAmount,
    #[error("Value overflowed its maximum value")]
    Overflow,
    #[error("Value from too small slice")]
    FromSliceTooSmall,
    #[error("Value from too big slice")]
    FromSliceTooBig,
}

impl Sum for Value {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Value::zero(), Self::saturating_add)
    }
}

impl ops::Add for Value {
    type Output = Result<Value, ValueError>;

    fn add(self, other: Value) -> Self::Output {
        self.checked_add(other)
    }
}

impl ops::Sub for Value {
    type Output = Result<Value, ValueError>;

    fn sub(self, other: Value) -> Self::Output {
        self.checked_sub(other)
    }
}

impl AsRef<u64> for Value {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl property::Deserialize for Value {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::new(reader);
        codec.get_u64().map(Value)
    }
}
impl Readable for Value {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        buf.get_u64().map(Value)
    }
}

impl property::Serialize for Value {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        let mut codec = Codec::new(writer);
        codec.put_u64(self.0)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = ValueError;
    fn try_from(slice: &[u8]) -> Result<Value, ValueError> {
        use std::cmp::Ordering::*;

        match slice.len().cmp(&VALUE_SERIALIZED_SIZE) {
            Less => Err(ValueError::FromSliceTooSmall),
            Greater => Err(ValueError::FromSliceTooBig),
            Equal => {
                let mut buf = [0u8; VALUE_SERIALIZED_SIZE];
                buf.copy_from_slice(slice);
                Ok(Value(u64::from_be_bytes(buf)))
            }
        }
    }
}

impl Into<Stake> for Value {
    fn into(self) -> Stake {
        Stake::from_value(self)
    }
}
