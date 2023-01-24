use bytekind::{ByteArray, Format, Plain};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

/// An ED25519 signature
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Arbitrary, Serialize, Deserialize)]
#[serde(bound = "ByteArray<F, 64>: Serialize + for<'a> Deserialize<'a>")]
pub struct Sig<F: Format = Plain>(pub ByteArray<F, 64>);

impl<F: Format> core::fmt::Debug for Sig<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Sig").field(&self.0).finish()
    }
}

impl<F: Format> Sig<F> {
    pub fn to_bytes(&self) -> [u8; 64] {
        *self.0.as_ref()
    }

    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes.into())
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0; 64];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

impl<F: Format> AsRef<[u8]> for Sig<F> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
