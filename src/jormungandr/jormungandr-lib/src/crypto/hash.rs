//! Hash interface for all that is a hash

use crate::crypto::serde as internal;
use chain_crypto::{
    digest::{Digest, DigestOf},
    hash::Blake2b256,
};
use chain_impl_mockchain::key;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// wrapper around the Blake2b256 hash
///
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Serialize, Deserialize)]
#[serde(remote = "key::Hash")]
pub struct Hash {
    #[serde(
        deserialize_with = "internal::deserialize_hash",
        serialize_with = "internal::serialize_hash",
        getter = "key::Hash::get_hash"
    )]
    hash: Blake2b256,
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        internal::serialize_hash(&self.hash, serializer)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            hash: internal::deserialize_hash(deserializer)?,
        })
    }
}

impl Hash {
    pub fn into_digest(self) -> Digest<Blake2b256> {
        self.hash.into()
    }

    pub fn into_digest_of<T>(self) -> DigestOf<Blake2b256, T> {
        self.into_digest().into()
    }

    pub fn to_hex(self) -> String {
        self.to_string()
    }

    pub fn from_hex(s: &str) -> Result<Self, chain_crypto::hash::Error> {
        s.parse().map(|hash| Hash { hash })
    }
}

/* ---------------- Display ------------------------------------------------ */

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.hash.fmt(f)
    }
}

impl FromStr for Hash {
    type Err = chain_crypto::hash::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Blake2b256::from_str(s).map(Hash::from)
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Hash")
            .field("0", &self.to_string())
            .finish()
    }
}

/* ---------------- AsRef -------------------------------------------------- */

impl AsRef<Blake2b256> for Hash {
    fn as_ref(&self) -> &Blake2b256 {
        &self.hash
    }
}

/* ---------------- Conversion --------------------------------------------- */

impl From<Blake2b256> for Hash {
    fn from(hash: Blake2b256) -> Self {
        Hash { hash }
    }
}

impl From<Hash> for Blake2b256 {
    fn from(hash: Hash) -> Self {
        hash.hash
    }
}

impl From<key::Hash> for Hash {
    fn from(hash: key::Hash) -> Self {
        Self {
            hash: hash.get_hash().clone(),
        }
    }
}

impl From<Hash> for key::Hash {
    fn from(hash: Hash) -> Self {
        Self::new(hash.hash)
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash { hash: bytes.into() }
    }
}

impl<T> From<DigestOf<Blake2b256, T>> for Hash {
    fn from(d: DigestOf<Blake2b256, T>) -> Hash {
        let x: [u8; 32] = d.into();
        x.into()
    }
}

impl<T> From<Hash> for DigestOf<Blake2b256, T> {
    fn from(h: Hash) -> Self {
        DigestOf::from(h.hash)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(hash: Hash) -> [u8; 32] {
        hash.hash.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen, TestResult};

    impl Arbitrary for Hash {
        fn arbitrary<G>(g: &mut G) -> Self
        where
            G: Gen,
        {
            Hash {
                hash: Blake2b256::arbitrary(g),
            }
        }
    }

    // test to check that hash is encoded in hexadecimal
    // when we use the Display trait
    #[test]
    fn hash_display() {
        const EXPECTED_HASH_STR: &str =
            "2020202020202020202020202020202020202020202020202020202020202020";
        const HASH_BYTES: [u8; 32] = [0x20; 32];

        let hash = Hash {
            hash: Blake2b256::from(HASH_BYTES),
        };

        assert_eq!(hash.to_string(), EXPECTED_HASH_STR);
    }

    // check that the hash is encoded with hexadecimal when utilising
    // serde with a human readable output
    #[test]
    fn hash_serde_human_readable() {
        const EXPECTED_HASH_STR: &str =
            "---\n\"2020202020202020202020202020202020202020202020202020202020202020\"\n";
        const HASH_BYTES: [u8; 32] = [0x20; 32];

        let hash = Hash {
            hash: Blake2b256::from(HASH_BYTES),
        };

        let hash_str = serde_yaml::to_string(&hash).unwrap();

        assert_eq!(hash_str, EXPECTED_HASH_STR);
    }

    quickcheck! {
        fn hash_display_and_from_str(hash: Hash) -> TestResult {
            let hash_str = hash.to_string();
            let hash_dec = match Hash::from_str(&hash_str) {
                Err(error) => return TestResult::error(error.to_string()),
                Ok(hash) => hash,
            };

            TestResult::from_bool(hash_dec == hash)
        }

        fn hash_serde_human_readable_encode_decode(hash: Hash) -> TestResult {
            let hash_str = serde_yaml::to_string(&hash).unwrap();
            let hash_dec : Hash= match serde_yaml::from_str(&hash_str) {
                Err(error) => return TestResult::error(error.to_string()),
                Ok(hash) => hash,
            };

            TestResult::from_bool(hash_dec == hash)
        }
    }
}
