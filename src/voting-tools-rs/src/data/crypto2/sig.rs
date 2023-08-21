use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

/// An ED25519 signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Arbitrary)]
pub struct Sig(pub [u8; 64]);

impl Serialize for Sig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = hex::encode(self); // TODO: stack allocate this string into a [u8; 128]
        String::serialize(&s, serializer)
    }
}

impl<'de> Deserialize<'de> for Sig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut bytes = [0; 64];
        hex::decode_to_slice(s.trim_start_matches("0x"), &mut bytes)
            .map_err(<D::Error as serde::de::Error>::custom)?;
        Ok(Self(bytes))
    }
}

impl Sig {
    /// Create a public key from a string slice containing hex bytes
    ///
    /// Will return an error if the string contains invalid hex, or doesn't contain exactly 32
    /// characters
    ///
    /// ```
    /// # use voting_tools_rs::Sig;
    /// let key = Sig::from_hex("0".repeat(128).as_str()).unwrap();
    /// assert_eq!(key, Sig([0; 64]));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if `s` is not a hex string representing an array of 64 bytes (i.e. a
    /// string with 128 chars)
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0; 64];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl AsRef<[u8]> for Sig {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
