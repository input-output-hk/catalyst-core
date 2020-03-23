use std::result;
use thiserror::Error;

/// Error regarding BIP39 operations
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Received an unsupported number of mnemonic words. The parameter
    /// contains the unsupported number. Supported values are
    /// described as part of the [`Type`](./enum.Type.html).
    #[error("Unsupported number of mnemonic words: {0}")]
    WrongNumberOfWords(usize),

    /// The entropy is of invalid size. The parameter contains the invalid size,
    /// the list of supported entropy size are described as part of the
    /// [`Type`](./enum.Type.html).
    #[error("Unsupported mnemonic entropy size: {0}")]
    WrongKeySize(usize),

    /// The given mnemonic is out of bound, i.e. its index is above 2048 and
    /// is invalid within BIP39 specifications.
    #[error("The given mnemonic is out of bound, {0}")]
    MnemonicOutOfBound(u16),

    /// Forward error regarding dictionary operations.
    #[error("Unknown mnemonic word")]
    LanguageError(
        #[source]
        #[from]
        crate::dictionary::Error,
    ),

    /// the Seed is of invalid size. The parameter is the given seed size,
    /// the expected seed size is [`SEED_SIZE`](./constant.SEED_SIZE.html).
    #[error("Invalid Seed Size, expected 64 bytes, but received {0} bytes.")]
    InvalidSeedSize(usize),

    /// checksum is invalid. The first parameter is the expected checksum,
    /// the second id the computed checksum. This error means that the given
    /// mnemonics are invalid to retrieve the original entropy. The user might
    /// have given an invalid mnemonic phrase.
    #[error("Invalid Entropy's Checksum, expected {0:08b} but found {1:08b}")]
    InvalidChecksum(u8, u8),
}

/// convenient Alias to wrap up BIP39 operations that may return
/// an [`Error`](./enum.Error.html).
pub type Result<T> = result::Result<T, Error>;
