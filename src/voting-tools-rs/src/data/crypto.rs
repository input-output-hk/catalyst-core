//! Types for cryptographic information used in the catalyst registration process

use core::fmt::{Debug, Display, Formatter};

use proptest::{
    arbitrary::StrategyFor,
    prelude::{any, Arbitrary},
    strategy::{Map, Strategy},
};
use serde::{de::Visitor, Deserialize, Serialize, Serializer};

use cardano_serialization_lib::chain_crypto::{
    ed25519::{Pub, Sig},
    AsymmetricPublicKey, Ed25519, VerificationAlgorithm,
};

/// Helper macro to write Debug and Display impls
macro_rules! fmt_impl {
    ($trait:ident, $t:ty) => {
        impl $trait for $t {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(&hex::encode(self.0.as_ref()))
            }
        }
    };
}

// we don't use `microtype` here because basically all the impls are custom

/// An ED25519 public key, which serializes and deserializes to/from a hex string
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicKeyHex(pub Pub);

fmt_impl!(Debug, PublicKeyHex);
fmt_impl!(Display, PublicKeyHex);

impl Serialize for PublicKeyHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(self.0.as_ref());
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for PublicKeyHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl Visitor<'_> for V {
            type Value = Pub;
            fn expecting(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str("a hex string representing an ED25519 public key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut bytes = [0; 32];
                match hex::decode_to_slice(v, &mut bytes) {
                    // this unwrap is safe because the slice is guaranteed to have 32 bytes
                    Ok(()) => Ok(Ed25519::public_from_binary(&bytes).unwrap()),
                    Err(e) => Err(E::custom(e.to_string())),
                }
            }
        }

        deserializer.deserialize_str(V).map(Self)
    }
}

impl Arbitrary for PublicKeyHex {
    type Parameters = ();
    type Strategy = Map<StrategyFor<[u8; 32]>, fn([u8; 32]) -> Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<[u8; 32]>().prop_map(|bytes| Self(Ed25519::public_from_binary(&bytes).unwrap()))
    }
}

impl PublicKeyHex {
    /// Convert this to the hex representation (without leading "0x")
    ///
    /// ```
    /// # use voting_tools_rs::PublicKeyHex;
    /// let sig = PublicKeyHex::from_bytes([0; 32]);
    ///
    /// assert_eq!(sig.to_string, "0".repeat(64));
    /// ```
    pub fn to_hex(&self) -> String {
        hex::encode(self.0.as_ref())
    }

    /// Create a signature from a byte array
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        // .unwrap() here is fine because it only panics if bytes.len() != 32
        Self(Ed25519::public_from_binary(&bytes).unwrap())
    }
}

/// An ED25519 signature that serializes and deserializes to/from a hex string
#[derive(Clone)]
pub struct SignatureHex(pub Sig);

fmt_impl!(Debug, SignatureHex);
fmt_impl!(Display, SignatureHex);

impl PartialEq for SignatureHex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl Serialize for SignatureHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = hex::encode(self.0.as_ref());
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for SignatureHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl Visitor<'_> for V {
            type Value = Sig;
            fn expecting(&self, f: &mut Formatter) -> core::fmt::Result {
                f.write_str("a hex string representing an ED25519 signature")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut bytes = [0; 64];
                match hex::decode_to_slice(v, &mut bytes) {
                    // this unwrap is safe because the slice is guaranteed to have 32 bytes
                    Ok(()) => Ok(Ed25519::signature_from_bytes(&bytes).unwrap()),
                    Err(e) => Err(E::custom(e.to_string())),
                }
            }
        }

        deserializer.deserialize_str(V).map(Self)
    }
}

impl Arbitrary for SignatureHex {
    type Parameters = ();
    type Strategy = Map<StrategyFor<[u8; 64]>, fn([u8; 64]) -> Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<[u8; 64]>().prop_map(|bytes| Self(Ed25519::signature_from_bytes(&bytes).unwrap()))
    }
}

impl SignatureHex {
    /// Convert this to the hex representation (without leading "0x")
    ///
    /// ```
    /// # use voting_tools_rs::SignatureHex;
    /// let sig = SignatureHex::from_bytes([0; 64]);
    ///
    /// assert_eq!(sig.to_string, "0".repeat(128));
    /// ```
    pub fn to_hex(&self) -> String {
        hex::encode(self.0.as_ref())
    }

    /// Create a signature from a byte array
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        // .unwrap() here is fine because it only panics if bytes.len() != 64
        Self(Ed25519::signature_from_bytes(&bytes).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use serde_json::json;
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn can_parse_cip15_vectors() {
        fn check_key(s: &str) {
            assert!(serde_json::from_value::<PublicKeyHex>(json!(s)).is_ok());
        }

        check_key("0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0");
        check_key("86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e");
        // check_key("e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef");
    }

    #[test]
    fn public_key_hex_fails_if_wrong_length() {
        let bytes = [0u8; 31];
        let string = hex::encode(&bytes);
        let json_string = serde_json::to_string(&json!(string)).unwrap();

        assert!(serde_json::from_str::<PublicKeyHex>(&json_string).is_err());

        let bytes = [0u8; 33];
        let string = hex::encode(&bytes);
        let json_string = serde_json::to_string(&json!(string)).unwrap();

        assert!(serde_json::from_str::<PublicKeyHex>(&json_string).is_err());
    }

    #[proptest]
    fn public_key_hex_serializes_deserializes_round_trip(key: PublicKeyHex) {
        let json = json!(key);
        let string = serde_json::to_string(&json).unwrap();
        let key_again: PublicKeyHex = serde_json::from_str(&string).unwrap();

        prop_assert_eq!(key, key_again);
    }

    #[test]
    fn signature_hex_fails_if_wrong_length() {
        let bytes = [0u8; 63];
        let string = hex::encode(&bytes);
        let json_string = serde_json::to_string(&json!(string)).unwrap();

        assert!(serde_json::from_str::<SignatureHex>(&json_string).is_err());

        let bytes = [0u8; 65];
        let string = hex::encode(&bytes);
        let json_string = serde_json::to_string(&json!(string)).unwrap();

        assert!(serde_json::from_str::<SignatureHex>(&json_string).is_err());
    }

    #[proptest]
    fn signature_hex_serializes_deserializes_round_trip(sig: SignatureHex) {
        let json = json!(sig);
        let string = serde_json::to_string(&json).unwrap();
        let sig_again: SignatureHex = serde_json::from_str(&string).unwrap();

        prop_assert_eq!(sig, sig_again);
    }
}
