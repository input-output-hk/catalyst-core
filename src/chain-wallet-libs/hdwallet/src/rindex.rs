//! random indexes wallet - 2 Level of randomly chosen hard derivation indexes Wallet

use crate::Key;
use chain_path_derivation::AnyScheme;
use ed25519_bip32::XPrv;

/// Implementation of 2 level randomly chosen derivation index wallet
///
/// This is for compatibility purpose with the existing 2 Level of
/// randomly chosen hard derivation indexes
/// wallet.
///
pub struct Wallet {
    root_key: Key<XPrv, AnyScheme>,
}
impl Wallet {
    pub fn from_root_key(root_key: Key<XPrv, AnyScheme>) -> Self {
        Wallet { root_key }
    }
}
