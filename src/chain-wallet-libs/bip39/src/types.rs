use crate::{Error, Result};
use std::{fmt, result, str};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseTypeError {
    #[error("Expecting a number")]
    NaN(
        #[source]
        #[from]
        std::num::ParseIntError,
    ),
    #[error("Not a valid number of mnemonic, expected one of [9, 12, 15, 18, 21, 24]")]
    InvalidNumber,
}

/// The support type of `Mnemonics`, i.e. the number of words supported in a
/// mnemonic phrase.
///
/// This enum provide the following properties:
///
/// | number of words | entropy size (bits) | checksum size (bits)  |
/// | --------------- | ------------------- | --------------------- |
/// | 9               | 96                  | 3                     |
/// | 12              | 128                 | 4                     |
/// | 15              | 160                 | 5                     |
/// | 18              | 192                 | 6                     |
/// | 21              | 224                 | 7                     |
/// | 24              | 256                 | 8                     |
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Type {
    Type9Words,
    Type12Words,
    Type15Words,
    Type18Words,
    Type21Words,
    Type24Words,
}
impl Type {
    pub fn from_word_count(len: usize) -> Result<Self> {
        match len {
            9 => Ok(Type::Type9Words),
            12 => Ok(Type::Type12Words),
            15 => Ok(Type::Type15Words),
            18 => Ok(Type::Type18Words),
            21 => Ok(Type::Type21Words),
            24 => Ok(Type::Type24Words),
            _ => Err(Error::WrongNumberOfWords(len)),
        }
    }

    pub fn from_entropy_size(len: usize) -> Result<Self> {
        match len {
            96 => Ok(Type::Type9Words),
            128 => Ok(Type::Type12Words),
            160 => Ok(Type::Type15Words),
            192 => Ok(Type::Type18Words),
            224 => Ok(Type::Type21Words),
            256 => Ok(Type::Type24Words),
            _ => Err(Error::WrongKeySize(len)),
        }
    }

    pub fn to_key_size(self) -> usize {
        match self {
            Type::Type9Words => 96,
            Type::Type12Words => 128,
            Type::Type15Words => 160,
            Type::Type18Words => 192,
            Type::Type21Words => 224,
            Type::Type24Words => 256,
        }
    }

    pub fn checksum_size_bits(self) -> usize {
        match self {
            Type::Type9Words => 3,
            Type::Type12Words => 4,
            Type::Type15Words => 5,
            Type::Type18Words => 6,
            Type::Type21Words => 7,
            Type::Type24Words => 8,
        }
    }

    pub fn mnemonic_count(self) -> usize {
        match self {
            Type::Type9Words => 9,
            Type::Type12Words => 12,
            Type::Type15Words => 15,
            Type::Type18Words => 18,
            Type::Type21Words => 21,
            Type::Type24Words => 24,
        }
    }
}

impl Default for Type {
    fn default() -> Type {
        Type::Type24Words
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Type9Words => 9.fmt(f),
            Type::Type12Words => 12.fmt(f),
            Type::Type15Words => 15.fmt(f),
            Type::Type18Words => 18.fmt(f),
            Type::Type21Words => 21.fmt(f),
            Type::Type24Words => 24.fmt(f),
        }
    }
}

impl str::FromStr for Type {
    type Err = ParseTypeError;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let i = s.parse()?;
        match i {
            9 => Ok(Type::Type9Words),
            12 => Ok(Type::Type12Words),
            15 => Ok(Type::Type15Words),
            18 => Ok(Type::Type18Words),
            21 => Ok(Type::Type21Words),
            24 => Ok(Type::Type24Words),
            _ => Err(ParseTypeError::InvalidNumber),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Type {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            use Type::*;
            const VALUES: &[Type] = &[
                Type9Words,
                Type12Words,
                Type15Words,
                Type18Words,
                Type21Words,
                Type24Words,
            ];
            let v = usize::arbitrary(g) % VALUES.len();
            VALUES[v]
        }
    }

    #[test]
    fn to_string() {
        assert_eq!(Type::Type9Words.to_string(), "9");
        assert_eq!(Type::Type12Words.to_string(), "12");
        assert_eq!(Type::Type15Words.to_string(), "15");
        assert_eq!(Type::Type18Words.to_string(), "18");
        assert_eq!(Type::Type21Words.to_string(), "21");
        assert_eq!(Type::Type24Words.to_string(), "24");
    }

    #[quickcheck]
    fn fmt_parse(t: Type) -> bool {
        let s = t.to_string();
        let v = s.parse::<Type>().unwrap();

        v == t
    }
}
