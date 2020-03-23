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

mod bits;
mod bip39;
mod error;
mod entropy;
mod seed;

pub use self::{
    bip39::*,
    error::{Error, Result},
    entropy::Entropy,
    seed::{Seed, SEED_SIZE},
};
