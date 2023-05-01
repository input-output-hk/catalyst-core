use std::fmt;

use chain_crypto::{Ed25519, PublicKey};
use jormungandr_lib::crypto::account::Identifier;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub struct IdentifierDef(pub Identifier);
pub struct VotingKeyVisitor;

impl Serialize for IdentifierDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("0x{}", self.0.to_hex()))
        } else {
            serializer.serialize_bytes(self.0.as_ref().as_ref())
        }
    }
}

impl<'de> Visitor<'de> for VotingKeyVisitor {
    type Value = Identifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a voting key as described in CIP-36")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Identifier::from_hex(v.trim_start_matches("0x"))
            .map_err(|e| E::custom(format!("invalid voting key {}: {}", v, e)))
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        <PublicKey<Ed25519>>::from_binary(v)
            .map_err(|e| E::custom(format!("invalid voting key: {}", e)))
            .map(Self::Value::from)
    }
}

impl<'de> Deserialize<'de> for IdentifierDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(VotingKeyVisitor).map(Self)
        } else {
            deserializer.deserialize_bytes(VotingKeyVisitor).map(Self)
        }
    }
}
