use crate::store::{States, Status, UtxoStore};
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    legacy::OldAddress,
    transaction::UtxoPointer,
};
use chain_path_derivation::rindex::{self, Rindex};
use ed25519_bip32::XPrv;
use hdkeygen::{rindex::AddressRecovering, Key};

pub struct Wallet {
    recovering: AddressRecovering,
    state: States<FragmentId, UtxoStore<Rindex<rindex::Address>>>,
}

impl Wallet {
    pub fn from_root_key(root_key: Key<XPrv, Rindex<rindex::Root>>) -> Self {
        Self {
            recovering: AddressRecovering::from_root_key(root_key),
            state: States::new(FragmentId::zero_hash(), UtxoStore::new()),
        }
    }

    /// get all the pending transactions of the wallet
    pub fn pending_transactions(&self) -> impl Iterator<Item = &FragmentId> {
        self.state.iter().filter_map(|(k, _, status)| {
            if status == Status::Pending {
                Some(k)
            } else {
                None
            }
        })
    }

    pub fn check_fragment(&mut self, fragment_id: &FragmentId, fragment: &Fragment) -> bool {
        let mut at_least_one_match = false;
        let (_, store, _) = self.state.last_state();
        let mut store = store.clone();
        if let Fragment::OldUtxoDeclaration(utxos) = fragment {
            for (output_index, (address, value)) in utxos.addrs.iter().enumerate() {
                let pointer = UtxoPointer {
                    transaction_id: *fragment_id,
                    output_index: output_index as u8,
                    value: *value,
                };

                if let Some(key) = self.check(address) {
                    at_least_one_match = true;
                    store = store.add(pointer, key);
                }
            }
        }
        self.state.push(*fragment_id, store);
        at_least_one_match
    }

    fn check(&mut self, address: &OldAddress) -> Option<Key<XPrv, Rindex<rindex::Address>>> {
        if let Some(derivation_path) = self.recovering.check_address(address) {
            Some(self.recovering.key(&derivation_path))
        } else {
            None
        }
    }
}
