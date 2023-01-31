use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::data::NetworkId;

/// An ED25519 public key
///
/// This is a wrapper around `[u8; 32]`, with serde impls that serialize to/from a hex string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary)]
pub struct PubKey(pub [u8; 32]);

impl Serialize for PubKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = hex::encode(self); // TODO: stack allocate this string into a [u8; 64]
        String::serialize(&s, serializer)
    }
}

impl<'de> Deserialize<'de> for PubKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut bytes = [0; 32];
        hex::decode_to_slice(&s, &mut bytes).map_err(<D::Error as serde::de::Error>::custom)?;
        Ok(Self(bytes))
    }
}

impl PubKey {
    /// Convert this to the hex representation (without leading "0x")
    ///
    /// ```
    /// # use voting_tools_rs::PubKey;
    /// let sig = PubKey::from_bytes([0; 32]);
    ///
    /// assert_eq!(sig.to_string, "0".repeat(64));
    /// ```
    #[inline]
    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }

    /// Create a public key from a string slice containing hex bytes
    ///
    /// Will return an error if the string contains invalid hex, or doesn't contain exactly 32
    /// characters
    ///
    /// ```
    /// # use voting_tools_rs::PubKey;
    /// let key = PubKey::from_str("0".repeat(64)).unwrap();
    /// assert_eq!(key, PubKey::from_bytes([0; 32]));
    /// ```
    #[inline]
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0; 32];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self(bytes))
    }

    /// Get the type (i.e. the top 4 bits of the leading byte)
    #[inline]
    pub fn ty(&self) -> u8 {
        let bytes: &[u8] = self.0.as_ref();
        // first byte, shift the top 4 bits to the bottom 4 bits
        bytes[0].wrapping_shr(4)
    }

    /// The network tag (0 = testnet, 1 = mainnet)
    ///
    /// Returns `None` if this is a byron/bootstrap address
    #[inline]
    pub fn network_id(&self) -> Option<NetworkId> {
        if self.ty() == 0b1000 {
            return None;
        }

        let bytes: &[u8] = self.0.as_ref();
        let lower_bits = bytes[0] & 0b0000_1111;

        match lower_bits {
            0 => Some(NetworkId::Testnet),
            1 => Some(NetworkId::Mainnet),
            _ => None,
        }
    }
}

impl AsRef<[u8]> for PubKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json, to_value};

    use super::*;

    #[test]
    fn can_deserialize_str() {
        #[derive(Deserialize, Serialize)]
        struct Foo {
            foo: PubKey,
        }

        let json = json!({"foo": "0".repeat(64)});
        let foo: Foo = from_value(json.clone()).unwrap();
        assert_eq!(foo.foo.0, [0; 32]);

        let json_again = to_value(&foo).unwrap();
        assert_eq!(json, json_again);
    }
}
