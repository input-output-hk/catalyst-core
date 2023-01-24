use bytekind::{ByteArray, Format, Plain};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::data::NetworkId;

/// An ED25519 public key
///
/// A `PubKey` is fundamentally just a `[u8; N]` with some extra type information. In particular,
/// it is generic over some `F` which implements [`Format`]. By specifying this parameter, you can
/// customize the serialize implementations
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, Serialize, Deserialize)]
#[serde(bound = "ByteArray<F, 32>: Serialize + for<'a> Deserialize<'a>")]
pub struct PubKey<F: Format = Plain>(pub ByteArray<F, 32>);

impl<F: Format> core::fmt::Debug for PubKey<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PubKey").field(&self.0).finish()
    }
}

impl<F: Format> PubKey<F> {
    pub fn to_bytes(&self) -> [u8; 32] {
        *self.0.as_ref()
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes.into())
    }

    /// Convert this to the hex representation (without leading "0x")
    ///
    /// ```
    /// # use voting_tools_rs::PublicKeyHex;
    /// let sig = PublicKeyHex::from_bytes([0; 32]);
    ///
    /// assert_eq!(sig.to_string, "0".repeat(64));
    /// ```
    #[inline]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Create a signature from a string slice containing hex bytes
    ///
    /// Will return an error if the string contains invalid hex, or doesn't contain exactly 64
    /// characters
    ///
    /// ```
    /// # use voting_tools_rs::PublicKeyHex;
    /// let key = PublicKeyHex::from_str("0".repeat(64)).unwrap();
    /// assert_eq!(key, PublicKeyHex::from_bytes([0; 32]));
    /// ```
    #[inline]
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0; 32];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self(bytes.into()))
    }

    /// Get the type (i.e. the top 4 bits of the leading byte)
    #[inline]
    pub fn ty(&self) -> u8 {
        let bytes: &[u8] = self.0.as_ref();
        bytes[0] >> 4
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
        let lower_bits = bytes[0] & 0b00001111;

        match lower_bits {
            0 => Some(NetworkId::Testnet),
            1 => Some(NetworkId::Mainnet),
            _ => None,
        }
    }
}

impl<F: Format> AsRef<[u8]> for PubKey<F> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
