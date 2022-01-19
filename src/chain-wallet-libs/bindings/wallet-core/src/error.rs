use std::{
    error,
    fmt::{self, Display},
    result,
};

/// result returned by a call, this allows to check if an error
/// occurred while executing the function.
///
/// if an error occurred it is then possible to collect more information
/// about the kind of error as well as details from the underlying libraries
/// that may be useful in case of bug reports or to figure out why the inputs
/// were not valid and resulted in the function to return with error
#[must_use = "ignoring this may result in ignoring an error"]
pub struct Result(result::Result<(), Error>);

/// The error structure, contains details of what may have gone wrong
///
/// See the error's kind for the high level information of what went wrong,
/// it is also possible to then extract details (if any) of the error.
///
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    details: Option<Box<dyn error::Error + Send + Sync + 'static>>,
}

/// a code representing the kind of error that occurred
#[repr(u32)]
pub enum ErrorCode {
    /// the input was not valid, it may be because it was a null pointer
    /// where it was expected to be already allocated. See the details
    /// for more information.
    ///
    /// When this kind of error occurs it is likely a developer flow issue
    /// rather than a user input issue. See the error details for more
    /// details.
    InvalidInput = 1,

    /// an error occurred while recovering a wallet
    WalletRecovering = 2,

    /// the operation on the wallet conversion object fail, it may be
    /// an out of bound operation when attempting to access the `nth`
    /// transaction of the conversion.
    WalletConversion = 3,

    /// the provided voting choice is out of the allowed range
    WalletVoteOutOfRange = 4,

    /// the wallet failed to build a valid transaction, for example
    /// not enough funds available
    WalletTransactionBuilding = 5,

    /// error encrypting or decrypting transfer protocol payload
    SymmetricCipherError = 6,

    /// authentication failed
    SymmetricCipherInvalidPassword = 7,

    /// vote encryption key is invalid
    /// either because is not valid bech32, or because of the underlying bytes
    InvalidVoteEncryptionKey = 8,

    /// wallet out of funds
    NotEnoughFunds = 9,

    /// invalid fragment
    InvalidFragment = 10,

    /// invalid transaction validity date, it's either before the current blockchain or after the
    /// maximum possible interval
    InvalidTransactionValidityDate = 11,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// kind of error where the input (named `argument_name`) is of
    /// invalid format or of unexpected value (null pointer).
    ///
    /// The `details` should provide more info on what caused the error.
    InvalidInput { argument_name: &'static str },

    /// an error occurred while recovering a wallet
    WalletRecovering,

    /// This is the kind of error that may happen when operating
    /// the transactions of the wallet conversion. For example,
    /// there may be an out of bound error
    WalletConversion,

    /// the provided voting choice is out of the allowed range
    WalletVoteOutOfRange,

    /// the wallet failed to build a valid transaction
    WalletTransactionBuilding,

    /// format error (malformed input, etc...)
    SymmetricCipherError,

    /// authentication failed
    SymmetricCipherInvalidPassword,

    /// vote encryption key is invalid
    /// either because is not valid bech32, or because of the underlying bytes
    InvalidVoteEncryptionKey,

    /// wallet out of funds
    NotEnoughFunds,

    /// invalid fragment
    InvalidFragment,

    /// invalid transaction validity date
    InvalidTransactionValidityDate,
}

impl ErrorKind {
    /// retrieve the error code associated to the error kind
    ///
    /// useful to extract the kind of error that occurred in a portable
    /// way, without to use string encoded version of the `ErrorKind`.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidInput { .. } => ErrorCode::InvalidInput,
            Self::WalletRecovering => ErrorCode::WalletRecovering,
            Self::WalletConversion => ErrorCode::WalletConversion,
            Self::WalletVoteOutOfRange => ErrorCode::WalletVoteOutOfRange,
            Self::WalletTransactionBuilding => ErrorCode::WalletTransactionBuilding,
            Self::SymmetricCipherError => ErrorCode::SymmetricCipherError,
            Self::SymmetricCipherInvalidPassword => ErrorCode::SymmetricCipherInvalidPassword,
            Self::InvalidVoteEncryptionKey => ErrorCode::InvalidVoteEncryptionKey,
            Self::NotEnoughFunds => ErrorCode::NotEnoughFunds,
            Self::InvalidFragment => ErrorCode::InvalidFragment,
            Self::InvalidTransactionValidityDate => ErrorCode::InvalidTransactionValidityDate,
        }
    }
}

impl Error {
    /// access the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// if there are details return the pointer to the error type that triggered
    /// the error.
    ///
    /// this is useful to display more details as to why an error occurred.
    pub fn details(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        self.details.as_ref().map(|boxed| boxed.as_ref())
    }

    /// construct a Result which is an error with invalid inputs
    ///
    /// `argument_name` is expected to be a pointer to the parameter name.
    ///
    /// # example
    ///
    /// ```
    /// # use wallet_core::{Result, Error};
    /// # use thiserror::Error;
    /// # #[derive(Error, Debug)]
    /// # #[error("Unexpected null pointer")]
    /// # struct NulPointer;
    /// fn example(pointer: *mut u8) -> Result {
    ///   if pointer.is_null() {
    ///     Error::invalid_input("pointer")
    ///         .with(NulPointer)
    ///         .into()
    ///   } else {
    ///     Result::success()
    ///   }
    /// }
    ///
    /// let result = example(std::ptr::null_mut());
    ///
    /// assert!(result.is_err());
    /// # assert!(!result.is_ok());
    /// ```
    pub fn invalid_input(argument_name: &'static str) -> Self {
        Self {
            kind: ErrorKind::InvalidInput { argument_name },
            details: None,
        }
    }

    pub fn wallet_recovering() -> Self {
        Self {
            kind: ErrorKind::WalletRecovering,
            details: None,
        }
    }

    pub fn wallet_conversion() -> Self {
        Self {
            kind: ErrorKind::WalletConversion,
            details: None,
        }
    }

    pub fn wallet_vote_range() -> Self {
        Self {
            kind: ErrorKind::WalletVoteOutOfRange,
            details: None,
        }
    }

    pub fn wallet_transaction() -> Self {
        Self {
            kind: ErrorKind::WalletTransactionBuilding,
            details: None,
        }
    }

    pub fn symmetric_cipher_error(err: symmetric_cipher::Error) -> Self {
        let kind = match err {
            symmetric_cipher::Error::AuthenticationFailed => {
                ErrorKind::SymmetricCipherInvalidPassword
            }
            _ => ErrorKind::SymmetricCipherError,
        };

        Self {
            kind,
            details: None,
        }
    }

    pub fn invalid_vote_encryption_key() -> Self {
        Self {
            kind: ErrorKind::InvalidVoteEncryptionKey,
            details: None,
        }
    }

    pub fn not_enough_funds() -> Self {
        Self {
            kind: ErrorKind::NotEnoughFunds,
            details: None,
        }
    }

    pub fn invalid_fragment() -> Self {
        Self {
            kind: ErrorKind::InvalidFragment,
            details: None,
        }
    }

    pub fn invalid_transaction_validity_date() -> Self {
        Self {
            kind: ErrorKind::InvalidTransactionValidityDate,
            details: None,
        }
    }

    /// set some details to the `Result` object if the `Result` is of
    /// error kind
    ///
    /// If the `Result` means success, then nothing is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use wallet_core::{Result, Error};
    /// # use thiserror::Error;
    /// # #[derive(Error, Debug)]
    /// # #[error("Unexpected null pointer")]
    /// # struct NulPointer;
    /// fn example(pointer: *mut u8) -> Result {
    ///   if pointer.is_null() {
    ///     Error::invalid_input("pointer").into()
    ///   } else {
    ///     Result::success()
    ///   }
    /// }
    ///
    /// let mut input = 2;
    /// let input: *mut u8 = &mut 2;
    /// let result = example(input).with(NulPointer);
    ///
    /// # assert!(!result.is_err());
    /// assert!(result.is_ok());
    /// ```
    ///
    pub fn with<E>(self, details: E) -> Self
    where
        E: error::Error + Send + Sync + 'static,
    {
        Self {
            details: Some(Box::new(details)),
            ..self
        }
    }
}

impl Result {
    /// returns `true` if the `Result` means success
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    /// returns `true` if the `Result` means error
    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }

    /// if it is an error, this function will returns the the error object,
    /// otherwise it will return `None`
    pub fn error(&self) -> Option<&Error> {
        self.0.as_ref().err()
    }

    /// constructor to build a `Result` that means success
    ///
    /// # example
    ///
    /// ```
    /// # use wallet_core::Result;
    ///
    /// let result = Result::success();
    ///
    /// assert!(result.is_ok());
    /// # assert!(!result.is_err());
    /// ```
    pub fn success() -> Self {
        Self(Ok(()))
    }

    /// set some details to the `Result` object if the `Result` is of
    /// error kind
    ///
    /// If the `Result` means success, then nothing is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use wallet_core::{Result, Error};
    /// # use thiserror::Error;
    /// # #[derive(Error, Debug)]
    /// # #[error("Unexpected null pointer")]
    /// # struct NulPointer;
    /// fn example(pointer: *mut u8) -> Result {
    ///   if pointer.is_null() {
    ///     Error::invalid_input("pointer").into()
    ///   } else {
    ///     Result::success()
    ///   }
    /// }
    ///
    /// let mut input = 2;
    /// let input: *mut u8 = &mut 2;
    /// let result = example(input).with(NulPointer);
    ///
    /// # assert!(!result.is_err());
    /// assert!(result.is_ok());
    /// ```
    ///
    pub fn with<E>(self, details: E) -> Self
    where
        E: error::Error + Send + Sync + 'static,
    {
        match self.0 {
            Ok(()) => Self::success(),
            Err(mut err) => {
                err.details = Some(Box::new(details));
                Self(Err(err))
            }
        }
    }

    pub fn into_c_api(self) -> *mut Error {
        match self.0 {
            Ok(()) => std::ptr::null_mut(),
            Err(err) => Box::into_raw(Box::new(err)),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { argument_name } => {
                write!(f, "The argument '{}' is invalid.", argument_name)
            }
            Self::WalletRecovering => f.write_str("Error while recovering a wallet"),
            Self::WalletConversion => {
                f.write_str("Error while performing operation on the wallet conversion object")
            }
            Self::WalletVoteOutOfRange => {
                f.write_str("The provided choice is out of the vote variants range")
            }
            Self::WalletTransactionBuilding => f.write_str(
                "Failed to build a valid transaction, probably not enough funds available",
            ),
            Self::SymmetricCipherError => f.write_str("malformed encryption or decryption payload"),
            Self::SymmetricCipherInvalidPassword => f.write_str("invalid decryption password"),
            Self::InvalidVoteEncryptionKey => f.write_str("invalid vote encryption key"),
            Self::NotEnoughFunds => f.write_str("not enough funds to create transaction"),
            Self::InvalidFragment => f.write_str("invalid fragment"),
            Self::InvalidTransactionValidityDate => {
                f.write_str("invalid transaction validity date")
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.details()
            .map(|trait_object| trait_object as &dyn error::Error)
    }
}

impl From<Error> for Result {
    fn from(error: Error) -> Self {
        Self(Err(error))
    }
}

impl From<result::Result<(), Error>> for Result {
    fn from(result: result::Result<(), Error>) -> Self {
        Self(result)
    }
}
