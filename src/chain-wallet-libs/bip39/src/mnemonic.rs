use zeroize::ZeroizeOnDrop;

use crate::{dictionary, Error, Result, Type};
use std::{fmt, ops::Deref, str};

/// the maximum authorized value for a mnemonic. i.e. 2047
pub const MAX_MNEMONIC_VALUE: u16 = 2047;

/// Safe representation of a valid mnemonic index (see
/// [`MAX_MNEMONIC_VALUE`](./constant.MAX_MNEMONIC_VALUE.html)).
///
/// See [`dictionary module documentation`](./dictionary/index.html) for
/// more details about how to use this.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MnemonicIndex(pub u16);

/// Language agnostic mnemonic phrase representation.
///
/// This is an handy intermediate representation of a given mnemonic
/// phrase. One can use this intermediate representation to translate
/// mnemonic from one [`Language`](./dictionary/trait.Language.html)
/// to another. **However** keep in mind that the [`Seed`](./struct.Seed.html)
/// is linked to the mnemonic string in a specific language, in a specific
/// dictionary. The [`Entropy`](./struct.Entropy.html) will be the same
/// but the resulted [`Seed`](./struct.Seed.html) will differ and all
/// the derived key of a HDWallet using the [`Seed`](./struct.Seed.html)
/// as a source to generate the root key.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Mnemonics(Vec<MnemonicIndex>);

/// Validated mnemonic words. This guarantee a given mnemonic phrase
/// has been safely validated against a dictionary.
///
/// See the module documentation for more details about how to use it
/// within the `chain_wallet` library.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, ZeroizeOnDrop)]
pub struct MnemonicString(String);

impl MnemonicString {
    /// create a `MnemonicString` from the given `String`. This function
    /// will validate the mnemonic phrase against the given [`Language`]
    ///
    /// [`Language`]: ./dictionary/trait.Language.html
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39::*;
    ///
    /// const MNEMONICS : &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let mnemonics = MnemonicString::new(&dictionary::ENGLISH, MNEMONICS.to_owned())
    ///     .expect("valid Mnemonic phrase");
    /// ```
    ///
    /// # Error
    ///
    /// This function may fail if one or all words are not recognized
    /// in the given [`Language`].
    ///
    pub fn new<D>(dic: &D, s: String) -> Result<Self>
    where
        D: dictionary::Language,
    {
        let _ = Mnemonics::from_string(dic, &s)?;

        Ok(MnemonicString(s))
    }
}

impl MnemonicIndex {
    /// smart constructor, validate the given value fits the mnemonic index
    /// boundaries (see [`MAX_MNEMONIC_VALUE`](./constant.MAX_MNEMONIC_VALUE.html)).
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39::*;
    /// #
    /// let index = MnemonicIndex::new(1029);
    /// assert!(index.is_ok());
    /// // this line will fail
    /// let index = MnemonicIndex::new(4029);
    /// assert_eq!(index, Err(Error::MnemonicOutOfBound(4029)));
    /// ```
    ///
    /// # Error
    ///
    /// returns an [`Error::MnemonicOutOfBound`](enum.Error.html#variant.MnemonicOutOfBound)
    /// if the given value does not fit the valid values.
    ///
    pub fn new(m: u16) -> Result<Self> {
        if m <= MAX_MNEMONIC_VALUE {
            Ok(MnemonicIndex(m))
        } else {
            Err(Error::MnemonicOutOfBound(m))
        }
    }

    /// lookup in the given dictionary to retrieve the mnemonic word.
    ///
    /// # panic
    ///
    /// this function may panic if the
    /// [`Language::lookup_word`](./dictionary/trait.Language.html#method.lookup_word)
    /// returns an error. Which should not happen.
    ///
    pub fn to_word<D>(self, dic: &D) -> String
    where
        D: dictionary::Language,
    {
        dic.lookup_word(self).unwrap()
    }

    /// retrieve the Mnemonic index from the given word in the
    /// given dictionary.
    ///
    /// # Error
    ///
    /// May fail with a [`LanguageError`](enum.Error.html#variant.LanguageError)
    /// if the given [`Language`](./dictionary/trait.Language.html) returns the
    /// given word is not within its dictionary.
    ///
    pub fn from_word<D>(dic: &D, word: &str) -> Result<Self>
    where
        D: dictionary::Language,
    {
        let v = dic.lookup_mnemonic(word)?;
        Ok(v)
    }
}

impl Mnemonics {
    /// get the [`Type`](./enum.Type.html) of this given `Mnemonics`.
    ///
    /// # panic
    ///
    /// the only case this function may panic is if the `Mnemonics` has
    /// been badly constructed (i.e. not from one of the given smart
    /// constructor).
    ///
    pub fn get_type(&self) -> Type {
        Type::from_word_count(self.0.len()).unwrap()
    }

    /// get the mnemonic string representation in the given
    /// [`Language`](./dictionary/trait.Language.html).
    ///
    pub fn to_string<D>(&self, dic: &D) -> MnemonicString
    where
        D: dictionary::Language,
    {
        let mut vec = String::new();
        let mut first = true;
        for m in self.0.iter() {
            if first {
                first = false;
            } else {
                vec.push_str(dic.separator());
            }
            vec.push_str(&m.to_word(dic))
        }
        MnemonicString(vec)
    }

    /// Construct the `Mnemonics` from its string representation in the given
    /// [`Language`](./dictionary/trait.Language.html).
    ///
    /// # Error
    ///
    /// May fail with a [`LanguageError`](enum.Error.html#variant.LanguageError)
    /// if the given [`Language`](./dictionary/trait.Language.html) returns the
    /// given word is not within its dictionary.
    ///
    pub fn from_string<D>(dic: &D, mnemonics: &str) -> Result<Self>
    where
        D: dictionary::Language,
    {
        let mut vec = vec![];
        for word in mnemonics.split(dic.separator()) {
            vec.push(MnemonicIndex::from_word(dic, word)?);
        }
        Mnemonics::from_mnemonics(vec)
    }

    /// Construct the `Mnemonics` from the given array of `MnemonicIndex`.
    ///
    /// # Error
    ///
    /// May fail if this is an invalid number of `MnemonicIndex`.
    ///
    pub fn from_mnemonics(mnemonics: Vec<MnemonicIndex>) -> Result<Self> {
        let _ = Type::from_word_count(mnemonics.len())?;
        Ok(Mnemonics(mnemonics))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &MnemonicIndex> {
        self.0.iter()
    }
}

impl AsRef<[MnemonicIndex]> for Mnemonics {
    fn as_ref(&self) -> &[MnemonicIndex] {
        &self.0[..]
    }
}

impl Deref for MnemonicString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl fmt::Display for MnemonicString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Drop for Mnemonics {
    fn drop(&mut self) {
        for byte in self.0.iter_mut() {
            *byte = MnemonicIndex(0);
        }
    }
}
