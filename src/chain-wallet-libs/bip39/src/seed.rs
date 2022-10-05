use crate::{Error, MnemonicString, Result};
use cryptoxide::hmac::Hmac;
use cryptoxide::pbkdf2::pbkdf2;
use cryptoxide::sha2::Sha512;
use std::ops::Deref;

/// the expected size of a seed, in bytes.
pub const SEED_SIZE: usize = 64;

/// A BIP39 `Seed` object, will be used to generate a given HDWallet
/// root key.
///
/// See the module documentation for more details about how to use it
/// within the `chain_wallet` library.
#[derive(zeroize::ZeroizeOnDrop)]
pub struct Seed([u8; SEED_SIZE]);

impl Seed {
    /// create a Seed by taking ownership of the given array
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::*;
    ///
    /// let bytes = [0u8;SEED_SIZE];
    /// let seed  = Seed::from_bytes(bytes);
    ///
    /// assert!(seed.as_ref().len() == SEED_SIZE);
    /// ```
    pub fn from_bytes(buf: [u8; SEED_SIZE]) -> Self {
        Seed(buf)
    }

    /// create a Seed by copying the given slice into a new array
    ///
    /// # Example
    ///
    /// ```
    /// use bip39::*;
    ///
    /// let bytes = [0u8;SEED_SIZE];
    /// let wrong = [0u8;31];
    ///
    /// assert!(Seed::from_slice(&wrong[..]).is_err());
    /// assert!(Seed::from_slice(&bytes[..]).is_ok());
    /// ```
    ///
    /// # Error
    ///
    /// This constructor may fail if the given slice's length is not
    /// compatible to define a `Seed` (see [`SEED_SIZE`](./constant.SEED_SIZE.html)).
    ///
    pub fn from_slice(buf: &[u8]) -> Result<Self> {
        if buf.len() != SEED_SIZE {
            return Err(Error::InvalidSeedSize(buf.len()));
        }
        let mut v = [0u8; SEED_SIZE];
        v[..].clone_from_slice(buf);
        Ok(Seed::from_bytes(v))
    }

    /// get the seed from the given [`MnemonicString`] and the given password.
    ///
    /// [`MnemonicString`]: ./struct.MnemonicString.html
    ///
    /// Note that the `Seed` is not generated from the `Entropy` directly. It is a
    /// design choice of Bip39.
    ///
    /// # Safety
    ///
    /// The password is meant to allow plausible deniability. While it is possible
    /// not to use a password to protect the HDWallet it is better to add one.
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39::*;
    ///
    /// const MNEMONICS : &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let mnemonics = MnemonicString::new(&dictionary::ENGLISH, MNEMONICS.to_owned())
    ///     .expect("valid Mnemonic phrase");
    ///
    /// let seed = Seed::from_mnemonic_string(&mnemonics, b"Bourbaki team rocks!");
    /// ```
    ///
    pub fn from_mnemonic_string(mnemonics: &MnemonicString, password: &[u8]) -> Self {
        let mut salt = Vec::from("mnemonic");
        salt.extend_from_slice(password);
        let mut mac = Hmac::new(Sha512::new(), mnemonics.as_bytes());
        let mut result = [0; SEED_SIZE];
        pbkdf2(&mut mac, &salt, 2048, &mut result);
        Self::from_bytes(result)
    }
}

impl PartialEq for Seed {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl AsRef<[u8]> for Seed {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Seed {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
