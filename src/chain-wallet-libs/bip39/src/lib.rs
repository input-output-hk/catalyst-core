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

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod bits;
mod entropy;
mod error;
mod mnemonic;
mod seed;
mod types;

pub mod dictionary;

pub use self::{
    entropy::Entropy,
    error::{Error, Result},
    mnemonic::{MnemonicIndex, MnemonicString, Mnemonics, MAX_MNEMONIC_VALUE},
    seed::{Seed, SEED_SIZE},
    types::Type,
};

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

    const TEST_VECTORS_ENGLISH: &[TestVector] = &include!("test_vectors/bip39_english.txt");
    const TEST_VECTORS_JAPANESE: &[TestVector] = &include!("test_vectors/bip39_japanese.txt");
}
