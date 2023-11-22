//! Address
//!
//! It uses a simple serialization format which is made to be concise:
//!
//! * First byte contains the discrimination information (1 bit) and the kind of address (7 bits)
//! * Remaining bytes contains a kind specific encoding describe after.
//!
//! 4 kinds of address are currently supported:
//!
//! * Single: Just a (spending) public key using the ED25519 algorithm
//! * Group: Same as single, but with a added (staking/group) public key
//!   using the ED25519 algorithm.
//! * Account: A account public key using the ED25519 algorithm
//! * Multisig: a multisig account public key
//!
//! Single key:
//!     DISCRIMINATION_BIT || SINGLE_KIND_TYPE (7 bits) || SPENDING_KEY
//!
//! Group key:
//!     DISCRIMINATION_BIT || GROUP_KIND_TYPE (7 bits) || SPENDING_KEY || ACCOUNT_KEY
//!
//! Account key:
//!     DISCRIMINATION_BIT || ACCOUNT_KIND_TYPE (7 bits) || ACCOUNT_KEY
//!
//! Multisig key:
//!     DISCRIMINATION_BIT || MULTISIG_KIND_TYPE (7 bits) || MULTISIG_MERKLE_ROOT_PUBLIC_KEY
//!
//! Script identifier:
//!     DISCRIMINATION_BIT || SCRIPT_KIND_TYPE (7 bits) || SCRIPT_IDENTIFIER
//!
//! Address human format is bech32 encoded

use bech32::{self, FromBase32, ToBase32};
use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};
use chain_crypto::{Ed25519, PublicKey, PublicKeyError};
use std::string::ToString;

#[cfg(any(test, feature = "property-test-api"))]
mod testing;
#[cfg(any(test, feature = "property-test-api"))]
use chain_crypto::testing::public_key_strategy;

// Allow to differentiate between address in
// production and testing setting, so that
// one type of address is not used in another setting.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, serde::Serialize, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub enum Discrimination {
    Production,
    Test,
}

/// Kind of an address, which include the possible variation of scheme
///
/// * Single address : just a single ed25519 spending public key
/// * Group address : an ed25519 spending public key followed by a group public key used for staking
/// * Account address : an ed25519 stake public key
/// * Multisig address : a multisig public key
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub enum Kind {
    Single(
        #[cfg_attr(any(test, feature = "property-test-api"), strategy(public_key_strategy::<Ed25519>()))]
         PublicKey<Ed25519>,
    ),
    Group(
        #[cfg_attr(any(test, feature = "property-test-api"), strategy(public_key_strategy::<Ed25519>()))]
         PublicKey<Ed25519>,
        #[cfg_attr(any(test, feature = "property-test-api"), strategy(public_key_strategy::<Ed25519>()))]
         PublicKey<Ed25519>,
    ),
    Account(
        #[cfg_attr(any(test, feature = "property-test-api"), strategy(public_key_strategy::<Ed25519>()))]
         PublicKey<Ed25519>,
    ),
    Multisig([u8; 32]),
    Script([u8; 32]),
}

/// Kind Type of an address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindType {
    Single,
    Group,
    Account,
    Multisig,
    Script,
}

/// Size of a Single address
pub const ADDR_SIZE_SINGLE: usize = 33;

/// Size of a Group address
pub const ADDR_SIZE_GROUP: usize = 65;

/// Size of an Account address
pub const ADDR_SIZE_ACCOUNT: usize = 33;

/// Size of an Multisig Account address
pub const ADDR_SIZE_MULTISIG: usize = 33;

/// Size of a script address
pub const ADDR_SIZE_SCRIPT: usize = 33;

const ADDR_KIND_LOW_SENTINEL: u8 = 0x2; /* anything under or equal to this is invalid */
pub const ADDR_KIND_SINGLE: u8 = 0x3;
pub const ADDR_KIND_GROUP: u8 = 0x4;
pub const ADDR_KIND_ACCOUNT: u8 = 0x5;
pub const ADDR_KIND_MULTISIG: u8 = 0x6;
pub const ADDR_KIND_SCRIPT: u8 = 0x7;
const ADDR_KIND_SENTINEL: u8 = 0x8; /* anything above or equal to this is invalid */

impl KindType {
    pub fn to_value(self) -> u8 {
        match self {
            KindType::Single => ADDR_KIND_SINGLE,
            KindType::Group => ADDR_KIND_GROUP,
            KindType::Account => ADDR_KIND_ACCOUNT,
            KindType::Multisig => ADDR_KIND_MULTISIG,
            KindType::Script => ADDR_KIND_SCRIPT,
        }
    }
}

/// An unstructured address including the
/// discrimination and the kind of address
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct Address(pub Discrimination, pub Kind);

impl Address {
    pub fn discrimination(&self) -> Discrimination {
        self.0
    }
    pub fn kind(&self) -> &Kind {
        &self.1
    }
}

#[derive(Debug)]
pub enum Error {
    EmptyAddress,
    InvalidKind,
    InvalidAddress,
    InvalidInternalEncoding,
    InvalidPrefix,
    MismatchPrefix,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::EmptyAddress => write!(f, "empty address"),
            Error::InvalidKind => write!(f, "invalid kind"),
            Error::InvalidAddress => write!(f, "invalid address"),
            Error::InvalidInternalEncoding => write!(f, "invalid internal encoding"),
            Error::InvalidPrefix => write!(f, "invalid prefix"),
            Error::MismatchPrefix => write!(f, "mismatch prefix"),
        }
    }
}
impl std::error::Error for Error {}

impl From<PublicKeyError> for Error {
    fn from(_: PublicKeyError) -> Error {
        Error::InvalidAddress
    }
}

impl From<bech32::Error> for Error {
    fn from(_: bech32::Error) -> Error {
        Error::InvalidInternalEncoding
    }
}

impl Address {
    /// Try to convert from_bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // check the kind is valid and length
        is_valid_data(bytes)?;

        let discr = get_discrimination_value(bytes[0]);
        let kind = match get_kind_value(bytes[0]) {
            ADDR_KIND_SINGLE => {
                let spending = PublicKey::from_binary(&bytes[1..])?;
                Kind::Single(spending)
            }
            ADDR_KIND_GROUP => {
                let spending = PublicKey::from_binary(&bytes[1..33])?;
                let group = PublicKey::from_binary(&bytes[33..])?;

                Kind::Group(spending, group)
            }
            ADDR_KIND_ACCOUNT => {
                let stake_key = PublicKey::from_binary(&bytes[1..])?;
                Kind::Account(stake_key)
            }
            ADDR_KIND_MULTISIG => {
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes[1..33]);
                Kind::Multisig(hash)
            }
            ADDR_KIND_SCRIPT => {
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes[1..33]);
                Kind::Script(hash)
            }
            _ => unreachable!(),
        };
        Ok(Address(discr, kind))
    }

    /// Return the size
    pub fn to_size(&self) -> usize {
        match self.1 {
            Kind::Single(_) => ADDR_SIZE_SINGLE,
            Kind::Group(_, _) => ADDR_SIZE_GROUP,
            Kind::Account(_) => ADDR_SIZE_ACCOUNT,
            Kind::Multisig(_) => ADDR_SIZE_MULTISIG,
            Kind::Script(_) => ADDR_SIZE_SCRIPT,
        }
    }

    /// Return the Kind type of a given address
    pub fn to_kind_type(&self) -> KindType {
        match self.1 {
            Kind::Single(_) => KindType::Single,
            Kind::Group(_, _) => KindType::Group,
            Kind::Account(_) => KindType::Account,
            Kind::Multisig(_) => KindType::Multisig,
            Kind::Script(_) => KindType::Script,
        }
    }

    fn to_kind_value(&self) -> u8 {
        self.to_kind_type().to_value()
    }

    /// Serialize an address into bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.serialize_as_vec()
            .expect("expect in memory allocation to always work")
    }

    /// create a base32 encoding of the byte serialization
    ///
    /// This is not the official normal human representation
    /// for the address, but is used for debug / other.
    pub fn base32(&self) -> String {
        let v = ToBase32::to_base32(&self.to_bytes());
        let alphabet = b"abcdefghijklmnopqrstuvwxyz234567";
        let mut out = Vec::new();
        for i in v {
            out.push(alphabet[i.to_u8() as usize])
        }
        unsafe { String::from_utf8_unchecked(out) }
    }

    pub fn public_key(&self) -> Option<&PublicKey<Ed25519>> {
        match self.1 {
            Kind::Single(ref pk) => Some(pk),
            Kind::Group(ref pk, _) => Some(pk),
            Kind::Account(ref pk) => Some(pk),
            Kind::Multisig(_) => None,
            Kind::Script(_) => None,
        }
    }
}

fn get_kind_value(first_byte: u8) -> u8 {
    first_byte & 0b0111_1111
}

fn get_discrimination_value(first_byte: u8) -> Discrimination {
    if (first_byte & 0b1000_0000) == 0 {
        Discrimination::Production
    } else {
        Discrimination::Test
    }
}

fn is_valid_data(bytes: &[u8]) -> Result<(Discrimination, KindType), Error> {
    if bytes.is_empty() {
        return Err(Error::EmptyAddress);
    }
    let kind_type = get_kind_value(bytes[0]);
    if kind_type <= ADDR_KIND_LOW_SENTINEL || kind_type >= ADDR_KIND_SENTINEL {
        return Err(Error::InvalidKind);
    }
    let kty = match kind_type {
        ADDR_KIND_SINGLE => {
            if bytes.len() != ADDR_SIZE_SINGLE {
                return Err(Error::InvalidAddress);
            }
            KindType::Single
        }
        ADDR_KIND_GROUP => {
            if bytes.len() != ADDR_SIZE_GROUP {
                return Err(Error::InvalidAddress);
            }
            KindType::Group
        }
        ADDR_KIND_ACCOUNT => {
            if bytes.len() != ADDR_SIZE_ACCOUNT {
                return Err(Error::InvalidAddress);
            }
            KindType::Account
        }
        ADDR_KIND_MULTISIG => {
            if bytes.len() != ADDR_SIZE_MULTISIG {
                return Err(Error::InvalidAddress);
            }
            KindType::Multisig
        }
        ADDR_KIND_SCRIPT => {
            if bytes.len() != ADDR_SIZE_SCRIPT {
                return Err(Error::InvalidAddress);
            }
            KindType::Script
        }
        _ => return Err(Error::InvalidKind),
    };
    Ok((get_discrimination_value(bytes[0]), kty))
}

/// A valid address in a human readable format
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressReadable(String);

impl AddressReadable {
    pub fn as_string(&self) -> &str {
        &self.0
    }

    pub fn get_prefix(&self) -> String {
        bech32::decode(&self.0)
            .expect("only valid bech32 string are accepted")
            .0
    }

    /// Validate from a String to create a valid AddressReadable
    pub fn from_string(expected_prefix: &str, s: &str) -> Result<Self, Error> {
        let (hrp, data, variant) = bech32::decode(s)?;
        if hrp != expected_prefix {
            return Err(Error::InvalidPrefix);
        };
        // We have to fix the address format to the original Bech32 encoding
        // for compatibility with tools and whatnot.
        if variant != bech32::Variant::Bech32 {
            return Err(Error::InvalidInternalEncoding);
        }
        let dat = Vec::from_base32(&data)?;
        let _ = is_valid_data(&dat[..])?;

        Ok(AddressReadable(s.to_string()))
    }

    pub fn from_str_anyprefix(s: &str) -> Result<Self, Error> {
        let (_, data, variant) = bech32::decode(s)?;
        // We have to fix the address format to the original Bech32 encoding
        // for compatibility with tools and whatnot.
        if variant != bech32::Variant::Bech32 {
            return Err(Error::InvalidInternalEncoding);
        }
        let dat = Vec::from_base32(&data)?;
        let _ = is_valid_data(&dat[..])?;

        Ok(AddressReadable(s.to_string()))
    }

    pub fn from_string_anyprefix(s: &str) -> Result<Self, Error> {
        Self::from_str_anyprefix(s)
    }

    /// Create a new AddressReadable from an encoded address
    pub fn from_address(prefix: &str, addr: &Address) -> Self {
        let v = ToBase32::to_base32(&addr.to_bytes());
        // Use the original Bech32 format from BIP-0173.
        // As long as the binary length of addresses is fixed, there is
        // no ambiguity in encoding.
        let r = bech32::encode(prefix, v, bech32::Variant::Bech32).unwrap();
        AddressReadable(r)
    }

    /// Convert a valid AddressReadable to an decoded address
    pub fn to_address(&self) -> Address {
        // the data has been verified ahead of time, so all unwrap are safe
        let (_, data, _variant) = bech32::decode(&self.0).unwrap();
        let dat = Vec::from_base32(&data).unwrap();
        Address::from_bytes(&dat[..]).unwrap()
    }
}

impl std::fmt::Display for AddressReadable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for AddressReadable {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        AddressReadable::from_string_anyprefix(s)
    }
}

impl Serialize for Address {
    fn serialized_size(&self) -> usize {
        Codec::u8_size()
            + match &self.1 {
                Kind::Single(spend) => spend.as_ref().len(),
                Kind::Group(spend, group) => spend.as_ref().len() + group.as_ref().len(),
                Kind::Account(stake_key) => stake_key.as_ref().len(),
                Kind::Multisig(hash) => hash.serialized_size(),
                Kind::Script(hash) => hash.serialized_size(),
            }
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        let first_byte = match self.0 {
            Discrimination::Production => self.to_kind_value(),
            Discrimination::Test => self.to_kind_value() | 0b1000_0000,
        };
        codec.put_u8(first_byte)?;
        match &self.1 {
            Kind::Single(spend) => codec.put_bytes(spend.as_ref())?,
            Kind::Group(spend, group) => {
                codec.put_bytes(spend.as_ref())?;
                codec.put_bytes(group.as_ref())?;
            }
            Kind::Account(stake_key) => codec.put_bytes(stake_key.as_ref())?,
            Kind::Multisig(hash) => codec.put_bytes(&hash[..])?,
            Kind::Script(hash) => codec.put_bytes(&hash[..])?,
        };

        Ok(())
    }

    fn serialize_as_vec(&self) -> Result<Vec<u8>, WriteError> {
        let mut data = Vec::with_capacity(self.to_size());
        self.serialize(&mut Codec::new(&mut data))?;
        Ok(data)
    }
}

fn chain_crypto_err(e: chain_crypto::PublicKeyError) -> ReadError {
    match e {
        PublicKeyError::SizeInvalid => {
            ReadError::StructureInvalid("publickey size invalid".to_string())
        }
        PublicKeyError::StructureInvalid => {
            ReadError::StructureInvalid("publickey structure invalid".to_string())
        }
    }
}

impl Deserialize for Address {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let byte = codec.get_u8()?;
        let discr = get_discrimination_value(byte);
        let kind = match get_kind_value(byte) {
            ADDR_KIND_SINGLE => {
                let bytes = <[u8; 32]>::deserialize(codec)?;
                let spending = PublicKey::from_binary(&bytes[..]).map_err(chain_crypto_err)?;
                Kind::Single(spending)
            }
            ADDR_KIND_GROUP => {
                let bytes = <[u8; 32]>::deserialize(codec)?;
                let spending = PublicKey::from_binary(&bytes[..]).map_err(chain_crypto_err)?;
                let bytes = <[u8; 32]>::deserialize(codec)?;
                let group = PublicKey::from_binary(&bytes[..]).map_err(chain_crypto_err)?;
                Kind::Group(spending, group)
            }
            ADDR_KIND_ACCOUNT => {
                let bytes = <[u8; 32]>::deserialize(codec)?;
                let stake_key = PublicKey::from_binary(&bytes[..]).map_err(chain_crypto_err)?;
                Kind::Account(stake_key)
            }
            ADDR_KIND_MULTISIG => {
                let bytes = <[u8; 32]>::deserialize(codec)?;
                Kind::Multisig(bytes)
            }
            ADDR_KIND_SCRIPT => {
                let bytes = <[u8; 32]>::deserialize(codec)?;
                Kind::Script(bytes)
            }
            n => return Err(ReadError::UnknownTag(n as u32)),
        };
        Ok(Address(discr, kind))
    }
}

/// error that can happen when parsing the Discrimination
/// from a string
#[derive(Debug)]
pub struct ParseDiscriminationError(String);
impl std::fmt::Display for ParseDiscriminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Invalid Address Discrimination `{}'. Expected `production' or `test'.",
            self.0
        )
    }
}
impl std::error::Error for ParseDiscriminationError {}

impl std::fmt::Display for Discrimination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Discrimination::Production => write!(f, "production"),
            Discrimination::Test => write!(f, "test"),
        }
    }
}
impl std::str::FromStr for Discrimination {
    type Err = ParseDiscriminationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "production" => Ok(Discrimination::Production),
            "test" => Ok(Discrimination::Test),
            _ => Err(ParseDiscriminationError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use proptest::prelude::*;
    use test_strategy::proptest;

    const TEST_PREFIX: &str = "ca";

    fn property_serialize_deserialize(addr: &Address) {
        let data = addr.to_bytes();
        let r = Address::from_bytes(&data[..]).unwrap();
        assert_eq!(&r, addr);
    }

    fn expected_base32(addr: &Address, expected: &'static str) {
        assert_eq!(addr.base32(), expected.to_string());
    }

    fn expected_bech32(addr: &Address, expected: &'static str) {
        assert_eq!(
            AddressReadable::from_address(TEST_PREFIX, addr),
            AddressReadable(expected.to_string())
        );
    }

    fn property_readable(addr: &Address) {
        let ar = AddressReadable::from_address(TEST_PREFIX, addr);
        let a = ar.to_address();
        let ar2 = AddressReadable::from_string(TEST_PREFIX, ar.as_string())
            .expect("address is readable from string");
        assert_eq!(addr, &a);
        assert_eq!(ar, ar2);
    }

    #[proptest]
    fn from_address_to_address(address: Address) {
        let readable = AddressReadable::from_address(TEST_PREFIX, &address);
        let decoded = readable.to_address();

        prop_assert_eq!(address, decoded);
    }

    #[proptest]
    fn to_bytes_from_bytes(address: Address) {
        let readable = address.to_bytes();
        let decoded = Address::from_bytes(&readable).unwrap();

        prop_assert_eq!(address, decoded);
    }

    #[test]
    fn unit_tests() {
        let fake_spendingkey: PublicKey<Ed25519> = PublicKey::from_binary(&[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ])
        .unwrap();
        let fake_groupkey: PublicKey<Ed25519> = PublicKey::from_binary(&[
            41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
            63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
        ])
        .unwrap();
        let fake_accountkey: PublicKey<Ed25519> = PublicKey::from_binary(&[
            41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
            63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
        ])
        .unwrap();

        {
            let addr = Address(
                Discrimination::Production,
                Kind::Single(fake_spendingkey.clone()),
            );
            property_serialize_deserialize(&addr);
            property_readable(&addr);
            expected_base32(
                &addr,
                "amaqeayeaudaocajbifqydiob4ibceqtcqkrmfyydenbwha5dypsa",
            );
            expected_bech32(
                &addr,
                "ca1qvqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jqxuzx4s",
            );
        }

        {
            let addr = Address(
                Discrimination::Production,
                Kind::Group(fake_spendingkey.clone(), fake_groupkey.clone()),
            );
            property_serialize_deserialize(&addr);
            property_readable(&addr);
            expected_bech32(&addr, "ca1qsqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jq2f29vkz6t30xqcnyve5x5mrwwpe8ganc0f78aqyzsjrg3z5v36gguhxny");
            expected_base32(&addr, "aqaqeayeaudaocajbifqydiob4ibceqtcqkrmfyydenbwha5dypsakjkfmwc2lrpgaytemzugu3doobzhi5typj6h5aecqsdircumr2i");
        }

        {
            let addr = Address(
                Discrimination::Test,
                Kind::Group(fake_spendingkey, fake_groupkey),
            );
            property_serialize_deserialize(&addr);
            property_readable(&addr);
            expected_bech32(&addr, "ca1ssqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jq2f29vkz6t30xqcnyve5x5mrwwpe8ganc0f78aqyzsjrg3z5v36gjdetkp");
            expected_base32(&addr, "qqaqeayeaudaocajbifqydiob4ibceqtcqkrmfyydenbwha5dypsakjkfmwc2lrpgaytemzugu3doobzhi5typj6h5aecqsdircumr2i");
        }

        {
            let addr = Address(Discrimination::Test, Kind::Account(fake_accountkey));
            property_serialize_deserialize(&addr);
            property_readable(&addr);
            expected_base32(
                &addr,
                "quusukzmfuxc6mbrgiztinjwg44dsor3hq6t4p2aifbegrcfizduq",
            );
            expected_bech32(
                &addr,
                "ca1s55j52ev95hz7vp3xgengdfkxuurjw3m8s7nu06qg9pyx3z9ger5samu4rv",
            );
        }
    }
}
