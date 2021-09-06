use bech32::{self, Error as Bech32Error, FromBase32, ToBase32};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

/// Bech32 encoding for fixed-size binary objects.
pub trait Bech32 {
    /// The human-readable prefix that is used to represent the
    /// the object in the Bech32 format. On decoding, the HRP of the input
    /// string is checked against this value.
    const BECH32_HRP: &'static str;

    /// Length of the encoded binary data.
    const BYTES_LEN: usize;

    /// Decodes the object from its Bech32 string representation.
    fn try_from_bech32_str(bech32_str: &str) -> Result<Self>
    where
        Self: Sized;

    /// Produces a Bech32 string format representation of the object.
    fn to_bech32_str(&self) -> String;
}

pub fn to_bech32_from_bytes<B: Bech32>(bytes: &[u8]) -> String {
    // As long as the size of the object is fixed, the original Bech32 format
    // described in BIP-0173 should produce unambiguous encoding.
    assert_eq!(
        bytes.len(),
        B::BYTES_LEN,
        "encoded binary length should be {} bytes",
        B::BYTES_LEN
    );
    bech32::encode(B::BECH32_HRP, bytes.to_base32(), bech32::Variant::Bech32)
        .unwrap_or_else(|e| panic!("Failed to build bech32: {}", e))
}

pub fn try_from_bech32_to_bytes<B: Bech32>(bech32_str: &str) -> Result<Vec<u8>> {
    let (hrp, data, _variant) = bech32::decode(bech32_str)?;
    if hrp != B::BECH32_HRP {
        return Err(Error::HrpInvalid {
            expected: B::BECH32_HRP,
            actual: hrp,
        });
    }
    let data = Vec::<u8>::from_base32(&data)?;
    if data.len() != B::BYTES_LEN {
        return Err(Error::UnexpectedDataLen {
            expected: B::BYTES_LEN,
            actual: data.len(),
        });
    }
    Ok(data)
}

#[derive(Debug)]
pub enum Error {
    Bech32Malformed(Bech32Error),
    HrpInvalid {
        expected: &'static str,
        actual: String,
    },
    DataInvalid(Box<dyn StdError + Send + Sync + 'static>),
    UnexpectedDataLen {
        expected: usize,
        actual: usize,
    },
}

impl Error {
    pub fn data_invalid(cause: impl StdError + Send + Sync + 'static) -> Self {
        Error::DataInvalid(Box::new(cause))
    }
}

impl From<Bech32Error> for Error {
    fn from(error: Bech32Error) -> Self {
        Error::Bech32Malformed(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        match self {
            Error::Bech32Malformed(_) => write!(f, "Failed to parse bech32, invalid data format"),
            Error::HrpInvalid { expected, actual } => write!(
                f,
                "Parsed bech32 has invalid HRP prefix '{}', expected '{}'",
                actual, expected
            ),
            Error::DataInvalid(_) => write!(f, "Failed to parse data decoded from bech32"),
            Error::UnexpectedDataLen { expected, actual } => write!(
                f,
                "parsed bech32 data has length {}, expected {}",
                actual, expected
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Bech32Malformed(cause) => Some(cause),
            Error::DataInvalid(cause) => Some(&**cause),
            _ => None,
        }
    }
}
