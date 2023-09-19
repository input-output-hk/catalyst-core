//! Defines the Voters Public Key
//!
use poem_openapi::{types::Example, NewType};
use serde::Deserialize;

/// A Voters Public ED25519 Key (as registered in their most recent valid
/// [CIP-15](https://cips.cardano.org/cips/cip15) or [CIP-36](https://cips.cardano.org/cips/cip36) registration).
#[derive(NewType, Deserialize)]
#[oai(example = true)]
pub(crate) struct VotingPublicKey(pub String);

impl Example for VotingPublicKey {
    fn example() -> Self {
        Self("0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663".into())
    }
}
