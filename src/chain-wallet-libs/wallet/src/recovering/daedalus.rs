use super::RecoveryError;
use crate::transaction::{Dump, WitnessBuilder};
use chain_impl_mockchain::{
    fragment::Fragment,
    legacy::OldAddress,
    transaction::{Input, UtxoPointer},
    value::Value,
};
use chain_path_derivation::rindex::{self, Rindex};
use ed25519_bip32::XPrv;
use hdkeygen::{
    rindex::{AddressRecovering, Wallet},
    Key,
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct RecoveringDaedalus {
    wallet: Wallet,
    address_recovering: AddressRecovering,
    value_total: Value,
    utxos: HashMap<UtxoPointer, Key<XPrv, Rindex<rindex::Address>>>,
}

impl RecoveringDaedalus {
    pub(crate) fn new(wallet: Wallet) -> Self {
        let address_recovering = wallet.address_recovering();
        Self {
            wallet,
            address_recovering,
            value_total: Value::zero(),
            utxos: HashMap::with_capacity(128),
        }
    }

    pub fn value_total(&self) -> Value {
        self.value_total
    }

    /// convenient function to parse a block and check for owned token
    pub fn check_fragments<'a>(
        &mut self,
        fragments: impl Iterator<Item = &'a Fragment>,
    ) -> Result<(), RecoveryError> {
        for fragment in fragments {
            self.check_fragment(fragment)?
        }
        Ok(())
    }

    pub fn check_fragment(&mut self, fragment: &Fragment) -> Result<(), RecoveryError> {
        let fragment_id = fragment.hash();
        if let Fragment::OldUtxoDeclaration(utxos) = fragment {
            for (output_index, (address, value)) in utxos.addrs.iter().enumerate() {
                let pointer = UtxoPointer {
                    transaction_id: fragment_id,
                    output_index: output_index as u8,
                    value: *value,
                };

                self.check(pointer, address)?;
            }
        }
        Ok(())
    }

    pub fn check_address(&self, address: &OldAddress) -> bool {
        self.address_recovering.check_address(address).is_some()
    }

    pub fn check(
        &mut self,
        pointer: UtxoPointer,
        address: &OldAddress,
    ) -> Result<(), RecoveryError> {
        if let Some(derivation_path) = self.address_recovering.check_address(address) {
            self.value_total = self.value_total.saturating_add(pointer.value);
            let key = self.wallet.key(&derivation_path);
            match self.utxos.entry(pointer) {
                Entry::Occupied(_entry) => return Err(RecoveryError::DuplicatedUtxo),
                Entry::Vacant(entry) => entry.insert(key),
            };
        }
        Ok(())
    }
    /// dump all the inputs
    pub fn dump_in(&self, dump: &mut Dump) {
        for (pointer, key) in self.utxos.iter() {
            dump.push(
                Input::from_utxo(*pointer),
                WitnessBuilder::OldUtxo {
                    xprv: key.as_ref().clone(),
                },
            )
        }
    }
}
