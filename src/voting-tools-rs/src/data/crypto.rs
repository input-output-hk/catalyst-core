//! Types for cryptographic information used in the catalyst registration process

use core::fmt::{Debug, Formatter};

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

/// An ED25519 public key, which serializes and deserializes to/from a hex string
#[derive(Clone, PartialEq)]
pub struct PublicKeyHex(pub Pub);

impl Debug for PublicKeyHex {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&hex::encode(self.0.as_ref()))
    }
}

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

/// An ED25519 signature that serializes and deserializes to/from a hex string
#[derive(Clone)]
pub struct SignatureHex(pub Sig);

impl PartialEq for SignatureHex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl Debug for SignatureHex {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&hex::encode(self.0.as_ref()))
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
#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use serde_json::json;
    use test_strategy::proptest;

    use super::*;

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
