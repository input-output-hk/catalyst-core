//! BIP39 mnemonics
//!
//! Can be used to generate the root key of a given HDWallet,
//! an address or simply convert bits to mnemonic for human friendly
//! value.
//!
//! For more details about the protocol, see
//! [Bitcoin Improvement Proposal 39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
//!
//! # Example
//!
//! ## To create a new HDWallet
//!
//! ```
//! # extern crate rand;
//! #
//! # use bip39::*;
//!
//! // first, you need to generate the original entropy
//! let entropy = Entropy::generate(Type::Type18Words, rand::random);
//!
//! // human readable mnemonics (in English) to retrieve the original entropy
//! // and eventually recover a HDWallet.
//! let mnemonic_phrase = entropy.to_mnemonics().to_string(&dictionary::ENGLISH);
//!
//! // The seed of the HDWallet is generated from the mnemonic string
//! // in the associated language.
//! let seed = Seed::from_mnemonic_string(&mnemonic_phrase, b"some password");
//! ```
//!
//! ## To recover a HDWallet
//!
//! ```
//! # use bip39::*;
//!
//! let mnemonics = "mimic left ask vacant toast follow bitter join diamond gate attend obey";
//!
//! // to retrieve the seed, you only need the mnemonic string,
//! // here we construct the `MnemonicString` by verifying the
//! // mnemonics are valid against the given dictionary (English here).
//! let mnemonic_phrase = MnemonicString::new(&dictionary::ENGLISH, mnemonics.to_owned())
//!     .expect("the given mnemonics are valid English words");
//!
//! // The seed of the HDWallet is generated from the mnemonic string
//! // in the associated language.
//! let seed = Seed::from_mnemonic_string(&mnemonic_phrase, b"some password");
//! ```
//!

use crate::{Error, Result};
use std::{fmt, ops::Deref, result, str};

/// RAII for validated mnemonic words. This guarantee a given mnemonic phrase
/// has been safely validated against a dictionary.
///
/// See the module documentation for more details about how to use it
/// within the `chain_wallet` library.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "generic-serialization", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "generic-serialization", derive(Serialize, Deserialize))]
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

    pub fn to_key_size(&self) -> usize {
        match self {
            &Type::Type9Words => 96,
            &Type::Type12Words => 128,
            &Type::Type15Words => 160,
            &Type::Type18Words => 192,
            &Type::Type21Words => 224,
            &Type::Type24Words => 256,
        }
    }

    pub fn checksum_size_bits(&self) -> usize {
        match self {
            &Type::Type9Words => 3,
            &Type::Type12Words => 4,
            &Type::Type15Words => 5,
            &Type::Type18Words => 6,
            &Type::Type21Words => 7,
            &Type::Type24Words => 8,
        }
    }

    pub fn mnemonic_count(&self) -> usize {
        match self {
            &Type::Type9Words => 9,
            &Type::Type12Words => 12,
            &Type::Type15Words => 15,
            &Type::Type18Words => 18,
            &Type::Type21Words => 21,
            &Type::Type24Words => 24,
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
            &Type::Type9Words => write!(f, "9"),
            &Type::Type12Words => write!(f, "12"),
            &Type::Type15Words => write!(f, "15"),
            &Type::Type18Words => write!(f, "18"),
            &Type::Type21Words => write!(f, "21"),
            &Type::Type24Words => write!(f, "24"),
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

impl AsRef<[MnemonicIndex]> for Mnemonics {
    fn as_ref(&self) -> &[MnemonicIndex] {
        &self.0[..]
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

pub mod dictionary {
    //! Language support for BIP39 implementations.
    //!
    //! We provide default dictionaries  for the some common languages.
    //! This interface is exposed to allow users to implement custom
    //! dictionaries.
    //!
    //! Because this module is part of the `chain_wallet` crate and that we
    //! need to keep the dependencies as small as possible we do not support
    //! UTF8 NFKD by default. Users must be sure to compose (or decompose)
    //! our output (or input) UTF8 strings.
    //!

    use std::{error, fmt, result};

    use super::MnemonicIndex;

    /// Errors associated to a given language/dictionary
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
    #[cfg_attr(feature = "generic-serialization", derive(Serialize, Deserialize))]
    pub enum Error {
        /// this means the given word is not in the Dictionary of the Language.
        MnemonicWordNotFoundInDictionary(String),
    }
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &Error::MnemonicWordNotFoundInDictionary(ref s) => {
                    write!(f, "Mnemonic word not found in dictionary \"{}\"", s)
                }
            }
        }
    }
    impl error::Error for Error {}

    /// wrapper for `dictionary` operations that may return an error
    pub type Result<T> = result::Result<T, Error>;

    /// trait to represent the the properties that needs to be associated to
    /// a given language and its dictionary of known mnemonic words.
    ///
    pub trait Language {
        fn name(&self) -> &'static str;
        fn separator(&self) -> &'static str;
        fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex>;
        fn lookup_word(&self, mnemonic: MnemonicIndex) -> Result<String>;
    }

    /// Default Dictionary basic support for the different main languages.
    /// This dictionary expect the inputs to have been normalized (UTF-8 NFKD).
    ///
    /// If you wish to implement support for non pre-normalized form you can
    /// create reuse this dictionary in a custom struct and implement support
    /// for [`Language`](./trait.Language.html) accordingly (_hint_: use
    /// [`unicode-normalization`](https://crates.io/crates/unicode-normalization)).
    ///
    pub struct DefaultDictionary {
        pub words: [&'static str; 2048],
        pub name: &'static str,
    }
    impl Language for DefaultDictionary {
        fn name(&self) -> &'static str {
            self.name
        }
        fn separator(&self) -> &'static str {
            " "
        }
        fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex> {
            match self.words.iter().position(|x| x == &word) {
                None => Err(Error::MnemonicWordNotFoundInDictionary(word.to_string())),
                Some(v) => {
                    Ok(
                        // it is safe to call unwrap as we guarantee that the
                        // returned index `v` won't be out of bound for a
                        // `MnemonicIndex` (DefaultDictionary.words is an array of 2048 elements)
                        MnemonicIndex::new(v as u16).unwrap(),
                    )
                }
            }
        }
        fn lookup_word(&self, mnemonic: MnemonicIndex) -> Result<String> {
            Ok(unsafe { self.words.get_unchecked(mnemonic.0 as usize) }).map(|s| String::from(*s))
        }
    }

    /// default English dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#wordlists)
    ///
    pub const ENGLISH: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_english.txt"),
        name: "english",
    };

    /// default French dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#french)
    ///
    pub const FRENCH: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_french.txt"),
        name: "french",
    };

    /// default Japanese dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#japanese)
    ///
    pub const JAPANESE: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_japanese.txt"),
        name: "japanese",
    };

    /// default Korean dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#japanese)
    ///
    pub const KOREAN: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_korean.txt"),
        name: "korean",
    };

    /// default chinese simplified dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#chinese)
    ///
    pub const CHINESE_SIMPLIFIED: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_chinese_simplified.txt"),
        name: "chinese-simplified",
    };
    /// default chinese traditional dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#chinese)
    ///
    pub const CHINESE_TRADITIONAL: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_chinese_traditional.txt"),
        name: "chinese-traditional",
    };

    /// default italian dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#italian)
    ///
    pub const ITALIAN: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_italian.txt"),
        name: "italian",
    };

    /// default spanish dictionary as provided by the
    /// [BIP39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md#spanish)
    ///
    pub const SPANISH: DefaultDictionary = DefaultDictionary {
        words: include!("bip39_spanish.txt"),
        name: "spanish",
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    use unicode_normalization::UnicodeNormalization;

    use crate::{dictionary::Language, Entropy, Seed};

    #[test]
    fn english_dic() {
        let dic = &dictionary::ENGLISH;

        assert_eq!(dic.lookup_mnemonic("abandon"), Ok(MnemonicIndex(0)));
        assert_eq!(dic.lookup_mnemonic("crack"), Ok(MnemonicIndex(398)));
        assert_eq!(dic.lookup_mnemonic("shell"), Ok(MnemonicIndex(1579)));
        assert_eq!(dic.lookup_mnemonic("zoo"), Ok(MnemonicIndex(2047)));

        assert_eq!(dic.lookup_word(MnemonicIndex(0)), Ok("abandon".to_string()));
        assert_eq!(dic.lookup_word(MnemonicIndex(398)), Ok("crack".to_string()));
        assert_eq!(
            dic.lookup_word(MnemonicIndex(1579)),
            Ok("shell".to_string())
        );
        assert_eq!(dic.lookup_word(MnemonicIndex(2047)), Ok("zoo".to_string()));
    }

    #[test]
    fn mnemonic_zero() {
        let entropy = Entropy::Entropy12([0; 16]);
        let mnemonics = entropy.to_mnemonics();
        let entropy2 = Entropy::from_mnemonics(&mnemonics).unwrap();
        assert_eq!(entropy.as_ref(), entropy2.as_ref());
    }

    #[test]
    fn mnemonic_7f() {
        let entropy = Entropy::Entropy12([0x7f; 16]);
        let mnemonics = entropy.to_mnemonics();
        let entropy2 = Entropy::from_mnemonics(&mnemonics).unwrap();
        assert_eq!(entropy.as_ref(), entropy2.as_ref());
    }

    #[test]
    fn from_mnemonic_to_mnemonic() {
        let entropy = Entropy::generate(Type::Type12Words, random);
        let mnemonics = entropy.to_mnemonics();
        let entropy2 = Entropy::from_mnemonics(&mnemonics).unwrap();
        assert_eq!(entropy.as_ref(), entropy2.as_ref());
    }

    #[derive(Debug)]
    struct TestVector {
        entropy: &'static str,
        mnemonics: &'static str,
        seed: &'static str,
        passphrase: &'static str,
    }

    fn mk_test<D: dictionary::Language>(test: &TestVector, dic: &D) {
        // decompose the UTF8 inputs before processing:
        let mnemonics: String = test.mnemonics.nfkd().collect();
        let passphrase: String = test.passphrase.nfkd().collect();

        let mnemonics_ref = Mnemonics::from_string(dic, &mnemonics).expect("valid mnemonics");
        let mnemonics_str = MnemonicString::new(dic, mnemonics).expect("valid mnemonics string");
        let entropy_ref = Entropy::from_slice(&hex::decode(test.entropy).unwrap())
            .expect("decode entropy from hex");
        let seed_ref =
            Seed::from_slice(&hex::decode(test.seed).unwrap()).expect("decode seed from hex");

        assert!(mnemonics_ref.get_type() == entropy_ref.get_type());

        assert!(entropy_ref.to_mnemonics() == mnemonics_ref);
        assert!(
            entropy_ref
                == Entropy::from_mnemonics(&mnemonics_ref)
                    .expect("retrieve entropy from mnemonics")
        );

        assert_eq!(
            seed_ref.as_ref(),
            Seed::from_mnemonic_string(&mnemonics_str, passphrase.as_bytes()).as_ref()
        );
    }

    fn mk_tests<D: dictionary::Language>(tests: &[TestVector], dic: &D) {
        for test in tests {
            mk_test(test, dic);
        }
    }

    #[test]
    fn test_vectors_english() {
        mk_tests(TEST_VECTORS_ENGLISH, &dictionary::ENGLISH)
    }
    #[test]
    fn test_vectors_japanese() {
        mk_tests(TEST_VECTORS_JAPANESE, &dictionary::JAPANESE)
    }

    const TEST_VECTORS_ENGLISH: &'static [TestVector] = &include!("test_vectors/bip39_english.txt");
    const TEST_VECTORS_JAPANESE: &'static [TestVector] =
        &include!("test_vectors/bip39_japanese.txt");
}
