use core::fmt::{Debug, Display, Formatter};
use serde::{de::Visitor, Deserialize, Serialize};
use test_strategy::Arbitrary;

/// A simple wrapper around some bytes that are serialized and deserialized to/from hex strings

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary)]
pub struct HexBytes(pub Vec<u8>);

impl Display for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl Visitor<'_> for V {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing hex encoded bytes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                hex::decode(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(V).map(HexBytes)
    }
}

impl AsRef<[u8]> for HexBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for HexBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn serializes_to_from_str(bytes: HexBytes) {
        let json_str = serde_json::to_string(&bytes).unwrap();
        let bytes_again: HexBytes = serde_json::from_str(&json_str).unwrap();

        prop_assert_eq!(bytes, bytes_again);
    }
}
