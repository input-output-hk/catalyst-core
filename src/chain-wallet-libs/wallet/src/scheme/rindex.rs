use crate::{
    scheme::on_tx_input,
    store::{States, Status, UtxoStore},
};
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    legacy::OldAddress,
    transaction::{InputEnum, UtxoPointer},
    value::Value,
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

    /// get the utxos of this given wallet
    pub fn utxos(&self) -> &UtxoStore<Rindex<rindex::Address>> {
        self.state.last_state().1
    }

    /// confirm a pending transaction
    ///
    /// to only do once it is confirmed a transaction is on chain
    /// and is far enough in the blockchain history to be confirmed
    /// as immutable
    ///
    pub fn confirm(&mut self, fragment_id: &FragmentId) {
        self.state.confirm(fragment_id)
    }

    /// get the confirmed value of the wallet
    pub fn confirmed_value(&self) -> Value {
        self.state.confirmed_state().1.total_value()
    }

    /// get the unconfirmed value of the wallet
    ///
    /// if `None`, it means there is no unconfirmed state of the wallet
    /// and the value can be known from `confirmed_value`.
    ///
    /// The returned value is the value we expect to see at some point on
    /// chain once all transactions are on chain confirmed.
    pub fn unconfirmed_value(&self) -> Option<Value> {
        let (k, s, _) = self.state.last_state();
        let (kk, _) = self.state.confirmed_state();

        if k == kk {
            None
        } else {
            Some(s.total_value())
        }
    }

    /// get all the pending transactions of the wallet
    ///
    /// If empty it means there's no pending transactions waiting confirmation
    ///
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
        let (_, legacy, _) = self.state.last_state();
        let mut store = legacy.clone();

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(utxos) => {
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
            _ => {
                on_tx_input(fragment, |input| {
                    if let InputEnum::UtxoInput(pointer) = input.to_enum() {
                        if let Some(spent) = store.remove(&pointer) {
                            at_least_one_match = true;
                            store = spent;
                        }
                    }
                });
                // No on_tx_output because legacy code is not meant to live in
                // the outputs of normal transactions
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
