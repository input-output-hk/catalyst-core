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

use thiserror::Error;

use crate::MnemonicIndex;

/// Errors associated to a given language/dictionary
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Error)]
pub enum Error {
    /// this means the given word is not in the Dictionary of the Language.
    #[error("Mnemonic word not found in dictionary \"{0}\"")]
    MnemonicWordNotFoundInDictionary(String),
}

/// trait to represent the the properties that needs to be associated to
/// a given language and its dictionary of known mnemonic words.
///
pub trait Language {
    fn name(&self) -> &'static str;
    fn separator(&self) -> &'static str;
    fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex, Error>;
    fn lookup_word(&self, mnemonic: MnemonicIndex) -> Result<String, Error>;
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
    fn lookup_mnemonic(&self, word: &str) -> Result<MnemonicIndex, Error> {
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
    fn lookup_word(&self, mnemonic: MnemonicIndex) -> Result<String, Error> {
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
