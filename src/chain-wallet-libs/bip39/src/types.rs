use crate::{Error, Result};
use std::{fmt, result, str};

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
        Type::Type18Words
    }
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Type9Words => write!(f, "9"),
            Type::Type12Words => write!(f, "12"),
            Type::Type15Words => write!(f, "15"),
            Type::Type18Words => write!(f, "18"),
            Type::Type21Words => write!(f, "21"),
            Type::Type24Words => write!(f, "24"),
        }
    }
}
impl str::FromStr for Type {
    type Err = &'static str;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        match s {
            "9" => Ok(Type::Type9Words),
            "12" => Ok(Type::Type12Words),
            "15" => Ok(Type::Type15Words),
            "18" => Ok(Type::Type18Words),
            "21" => Ok(Type::Type21Words),
            "24" => Ok(Type::Type24Words),
            _ => Err("Unknown bip39 mnemonic size"),
        }
    }
}
