//! random indexes wallet - 2 Level of randomly chosen hard derivation indexes Wallet

mod hdpayload;

use crate::Key;
use cardano_legacy_address::{Addr, AddressMatchXPub, ExtendedAddr};
use chain_path_derivation::{AnyScheme, DerivationPath};
use ed25519_bip32::XPrv;
use hdpayload::{decode_derivation_path, HDKey};

/// Implementation of 2 level randomly chosen derivation index wallet
///
/// This is for compatibility purpose with the existing 2 Level of
/// randomly chosen hard derivation indexes
/// wallet.
///
pub struct Wallet {
    root_key: Key<XPrv, AnyScheme>,
}

/// check ownership of addresses with the linked wallet
pub struct AddressRecovering {
    root_key: Key<XPrv, AnyScheme>,
    payload_key: HDKey,
}

impl Wallet {
    pub fn from_root_key(root_key: Key<XPrv, AnyScheme>) -> Self {
        Wallet { root_key }
    }

    /// get an address recovering object, this object can be used to check the
    /// ownership of addresses
    pub fn address_recovering(&self) -> AddressRecovering {
        let payload_key = HDKey::new(self.root_key.public().public_key());

        AddressRecovering {
            root_key: self.root_key.clone(),
            payload_key,
        }
    }
}

impl AddressRecovering {
    /// check a legacy address is part of this wallet and returns the associated
    /// derived path.
    ///
    pub fn check_address(&self, addr: &Addr) -> Option<DerivationPath<AnyScheme>> {
        let extended = addr.deconstruct();
        let dp = self.derivation_path(&extended)?;

        let key_xprv = self
            .root_key
            .derive_path_unchecked::<AnyScheme, _>(dp.iter());
        let key_xpub = key_xprv.public();

        if addr.identical_with_xpub(key_xpub.public_key()) == AddressMatchXPub::Yes {
            Some(dp)
        } else {
            None
        }
    }

    /// retrieve the derivation path from the extended address if possible
    ///
    /// if there is no derivation path, maybe this is a bip44 address
    /// if it is not possible to decrypt the payload it is not associated
    /// to this wallet
    fn derivation_path(&self, address: &ExtendedAddr) -> Option<DerivationPath<AnyScheme>> {
        let payload = address
            .attributes
            .derivation_path
            .as_ref()
            .map(|p| p.as_slice())?;
        self.decode_payload(payload)
    }

    /// decode the payload expecting to retrieve the derivation path
    /// encrypted and encoded in cbor
    fn decode_payload(&self, payload: &[u8]) -> Option<DerivationPath<AnyScheme>> {
        let payload = self.payload_key.decrypt(payload).ok()?;

        decode_derivation_path(&payload)
    }
}
