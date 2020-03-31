//! module for all the recovering mechanism around the cardano blockchains

use crate::{keygen, Key};
use chain_path_derivation::AnyScheme;
use cryptoxide::digest::Digest;
use ed25519_bip32::{self, DerivationScheme, XPrv, XPRV_SIZE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecoveringError {
    #[error("Invalid BIP39 mnemonics")]
    Bip39Error(
        #[source]
        #[from]
        bip39::Error,
    ),
    #[error("CBOR Encoding error")]
    CBorEncoding(
        #[source]
        #[from]
        cbor_event::Error,
    ),
}

/// Compatibility with daedalus mnemonic addresses
///
/// > 2 Level of randomly chosen hard derivation indexes wallets
/// > uses the bip39 mnemonics but do not follow
/// > the whole BIP39 specifications.
///
/// # considerations
///
/// It is recommended to avoid using it as this is a weak
/// cryptographic scheme:
///
/// 1. it does not allow for mnemonic passwords (no plausible deniability);
/// 2. the use of an invariant of the cryptographic scheme makes it less
///
/// # internals
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
) -> Result<Key<XPrv, AnyScheme>, RecoveringError>
where
    D: bip39::dictionary::Language,
{
    let mnemonics = bip39::Mnemonics::from_string(dic, mnemonics_phrase)?;
    let entropy = bip39::Entropy::from_mnemonics(&mnemonics)?;

    let entropy_bytes = cbor_event::Value::Bytes(Vec::from(entropy.as_ref()));
    let entropy_cbor = cbor_event::cbor!(&entropy_bytes)?;
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
    let key = Key::new_unchecked(xprv, Default::default(), derivation_scheme);
    Ok(key)
}

/// method to recover the private key from bip39 mnemonics
///
/// this is the method used in yoroiwallet.com
pub fn from_bip39_mnemonics<D>(
    derivation_scheme: DerivationScheme,
    dic: &D,
    mnemonics_phrase: impl AsRef<str>,
    password: impl AsRef<[u8]>,
) -> Result<Key<XPrv, AnyScheme>, RecoveringError>
where
    D: bip39::dictionary::Language,
{
    let mnemonics = bip39::Mnemonics::from_string(dic, mnemonics_phrase.as_ref())?;
    let entropy = bip39::Entropy::from_mnemonics(&mnemonics)?;

    let mut seed = [0u8; XPRV_SIZE];
    keygen::generate_seed(&entropy, password.as_ref(), &mut seed);
    let xprv = XPrv::normalize_bytes_force3rd(seed);

    let key = Key::new_unchecked(xprv, Default::default(), derivation_scheme);
    Ok(key)
}

/// for some unknown design reasons Daedalus seeds are encoded in cbor
/// We then expect the input here to be cbor encoded before hand.
///
pub(crate) fn generate_from_daedalus_seed(bytes: &[u8]) -> XPrv {
    use cryptoxide::{hmac::Hmac, mac::Mac, sha2::Sha512};

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
        let xprv = XPrv::from_nonextended_force(&sk, &cc);

        // check if we find a good candidate
        if xprv.as_ref()[31] & 0x20 == 0 {
            return xprv;
        }

        iter += 1;
    }
}
