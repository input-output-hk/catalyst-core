//! random indexes wallet - 2 Level of randomly chosen hard derivation indexes Wallet

use bip39;
use cbor_event;
use cryptoxide;
use cryptoxide::digest::Digest;
use ed25519_bip32::{self, DerivationScheme, XPrv, XPub};
use std::{fmt, ops::Deref};

use super::scheme;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "generic-serialization", derive(Serialize, Deserialize))]
pub struct Addressing(u32, u32);
impl Addressing {
    pub fn new(account: u32, index: u32) -> Self {
        Addressing(account, index)
    }
}
impl ::std::fmt::Display for Addressing {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

/// Implementation of 2 level randomly chosen derivation index wallet
///
/// This is for compatibility purpose with the existing 2 Level of
/// randomly chosen hard derivation indexes
/// wallet.
///
pub struct Wallet {
    root_key: RootKey,
}
impl Wallet {
    pub fn from_root_key(root_key: RootKey) -> Self {
        Wallet { root_key }
    }

    /// Compatibility with daedalus mnemonic addresses
    ///
    /// > 2 Level of randomly chosen hard derivation indexes wallets uses the bip39 mnemonics but do not follow
    /// > the whole BIP39 specifications;
    ///
    /// 1. the mnemonic words are used to retrieve the entropy;
    /// 2. the entropy is serialized in CBOR;
    /// 3. the cbor serialised entropy is then hashed with Blake2b 256;
    /// 4. the blake2b digest is serialised in cbor;
    /// 5. the cbor serialised digest is then fed into HMAC sha256
    ///
    /// There are many things that can go wrong when implementing this
    /// process, it is all done correctly by this function: prefer using
    /// this function.
    pub fn from_daedalus_mnemonics<D>(
        derivation_scheme: DerivationScheme,
        dic: &D,
        mnemonics_phrase: &str,
    ) -> Result<Self>
    where
        D: bip39::dictionary::Language,
    {
        let root_key = RootKey::from_daedalus_mnemonics(derivation_scheme, dic, mnemonics_phrase)?;
        Ok(Wallet::from_root_key(root_key))
    }
}

impl Deref for Wallet {
    type Target = RootKey;
    fn deref(&self) -> &Self::Target {
        &self.root_key
    }
}

impl scheme::Wallet for Wallet {
    /// 2 Level of randomly chosen hard derivation indexes does not support Account model. Only one account: the root key.
    type Account = RootKey;
    /// 2 Level of randomly chosen hard derivation indexes does not support Account model. Only one account: the root key.
    type Accounts = Self::Account;
    /// 2 Level of randomly chosen hard derivation indexes derivation consists of 2 level of hard derivation, this is why
    /// it is not possible to have a public key account like in the bip44 model.
    type Addressing = Addressing;

    fn create_account(&mut self, _: &str, _: u32) -> Self::Account {
        self.root_key.clone()
    }
    fn list_accounts<'a>(&'a self) -> &'a Self::Accounts {
        &self.root_key
    }
}
impl scheme::Account for RootKey {
    type Addressing = Addressing;
    type PublicKey = XPub;

    fn generate_key<'a, I>(&'a self, addresses: I) -> Vec<Self::PublicKey>
    where
        I: Iterator<Item = &'a Self::Addressing>,
    {
        let (hint_low, hint_max) = addresses.size_hint();
        let mut vec = Vec::with_capacity(hint_max.unwrap_or(hint_low));

        for addressing in addresses {
            let xpub = self
                .root_key
                .derive(self.derivation_scheme, addressing.0)
                .derive(self.derivation_scheme, addressing.1)
                .public();
            vec.push(xpub);
        }
        vec
    }
}

#[derive(Debug)]
pub enum Error {
    Bip39Error(bip39::Error),
    DerivationError(ed25519_bip32::DerivationError),
    CBorEncoding(cbor_event::Error), // Should not happen really

    /// the addressing decoded in the payload is invalid
    InvalidPayloadAddressing(Vec<u32>),
}
impl From<bip39::Error> for Error {
    fn from(e: bip39::Error) -> Self {
        Error::Bip39Error(e)
    }
}
impl From<cbor_event::Error> for Error {
    fn from(e: cbor_event::Error) -> Self {
        Error::CBorEncoding(e)
    }
}
impl From<ed25519_bip32::DerivationError> for Error {
    fn from(e: ed25519_bip32::DerivationError) -> Self {
        Error::DerivationError(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Bip39Error(_) => write!(f, "Wallet's Mnemonic Error"),
            Error::DerivationError(_) => write!(f, "Invalid key derivation"),
            Error::CBorEncoding(_) => write!(f, "Error while encoding address in binary format"),
            Error::InvalidPayloadAddressing(path) => write!(
                f,
                "Payload has been decoded but is corrupted or of unexpected format (path: {:?})",
                path
            ),
        }
    }
}
/*
impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::Bip39Error(ref err) => Some(err),
            Error::DerivationError(ref err) => Some(err),
            Error::CBorEncoding(ref err) => Some(err),
            Error::InvalidPayloadAddressing(_) => None,
        }
    }
}
*/

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone)]
pub struct RootKey {
    root_key: XPrv,
    derivation_scheme: DerivationScheme,
}
impl RootKey {
    pub fn new(root_key: XPrv, derivation_scheme: DerivationScheme) -> Self {
        RootKey {
            root_key,
            derivation_scheme,
        }
    }
    pub fn from_daedalus_mnemonics<D>(
        derivation_scheme: DerivationScheme,
        dic: &D,
        mnemonics_phrase: &str,
    ) -> Result<Self>
    where
        D: bip39::dictionary::Language,
    {
        let mnemonics = bip39::Mnemonics::from_string(dic, mnemonics_phrase)?;
        let entropy = bip39::Entropy::from_mnemonics(&mnemonics)?;

        let entropy_bytes = cbor_event::Value::Bytes(Vec::from(entropy.as_ref()));
        let entropy_cbor = cbor!(&entropy_bytes)?;
        let seed: Vec<u8> = {
            let mut blake2b = cryptoxide::blake2b::Blake2b::new(32);
            blake2b.input(&entropy_cbor);
            let mut out = [0; 32];
            blake2b.result(&mut out);
            let mut se = cbor_event::se::Serializer::new_vec();
            se.write_bytes(&Vec::from(&out[..]))?;
            se.finalize()
        };

        let xprv = generate_from_daedalus_seed(&seed);
        Ok(RootKey::new(xprv, derivation_scheme))
    }

    /// Converts into the inner `XPrv` value
    pub fn into_xprv(self) -> XPrv {
        self.root_key
    }
}
impl Deref for RootKey {
    type Target = XPrv;
    fn deref(&self) -> &Self::Target {
        &self.root_key
    }
}

/// for some unknown design reasons Daedalus seeds are encoded in cbor
/// We then expect the input here to be cbor encoded before hande.
///
fn generate_from_daedalus_seed(bytes: &[u8]) -> XPrv {
    use cryptoxide::hmac::Hmac;
    use cryptoxide::mac::Mac;
    use cryptoxide::sha2::Sha512;

    let mut mac = Hmac::new(Sha512::new(), bytes);

    let mut iter = 1;

    loop {
        let s = format!("Root Seed Chain {}", iter);
        mac.reset();
        mac.input(s.as_bytes());
        let mut block = [0u8; 64];
        mac.raw_result(&mut block);

        let mut sk = [0; 32];
        sk.clone_from_slice(&block.as_ref()[0..32]);
        let mut cc = [0; 32];
        cc.clone_from_slice(&block.as_ref()[32..64]);
        let xprv = XPrv::from_nonextended(sk, cc);

        // check if we find a good candidate
        if xprv.as_ref()[31] & 0x20 == 0 {
            return xprv;
        }

        iter = iter + 1;
    }
}
