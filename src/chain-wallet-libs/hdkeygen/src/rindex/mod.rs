//! random indexes wallet - 2 Level of randomly chosen hard derivation indexes Wallet

mod hdpayload;

use crate::Key;
use chain_path_derivation::{
    rindex::{self, Rindex},
    DerivationPath,
};
use ed25519_bip32::XPrv;
pub use hdpayload::{decode_derivation_path, HdKey};

impl Key<XPrv, Rindex<rindex::Root>> {
    pub fn key(
        &self,
        derivation_path: &DerivationPath<Rindex<rindex::Address>>,
    ) -> Key<XPrv, Rindex<rindex::Address>> {
        self.derive_path_unchecked(derivation_path)
    }

    /// get an address recovering object, this object can be used to check the
    /// ownership of addresses
    pub fn hd_key(&self) -> HdKey {
        HdKey::new(self.public().public_key())
    }
}
