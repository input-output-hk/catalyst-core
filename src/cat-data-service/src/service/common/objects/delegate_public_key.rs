//! Defines the Public Key used by a Delegate.
//!
use poem_openapi::{types::Example, Object};

/// A Delegate Public ED25519 Key (as registered in their most recent valid
/// [CIP-36](https://cips.cardano.org/cips/cip36) registration).
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct DelegatePublicKey {
    #[oai(validator(pattern = "0x[0-9a-f]{64}"))]
    address: String,
}

impl From<String> for DelegatePublicKey {
    fn from(address: String) -> Self {
        Self { address }
    }
}

impl Example for DelegatePublicKey {
    fn example() -> Self {
        Self {
            address: "0xad4b948699193634a39dd56f779a2951a24779ad52aa7916f6912b8ec4702cee"
                .to_string(),
        }
    }
}
