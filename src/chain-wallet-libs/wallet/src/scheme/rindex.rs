use crate::{
    scheme::on_tx_input,
    states::{States, Status},
    store::UtxoStore,
};
use cardano_legacy_address::{AddressMatchXPub, ExtendedAddr};
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    legacy::OldAddress,
    transaction::{InputEnum, UtxoPointer},
    value::Value,
};
use chain_path_derivation::{
    rindex::{self, Rindex},
    DerivationPath,
};
use ed25519_bip32::XPrv;
use hdkeygen::{
    rindex::{decode_derivation_path, HDKey},
    Key,
};

pub struct Wallet {
    root_key: Key<XPrv, Rindex<rindex::Root>>,
    payload_key: HDKey,
    state: States<FragmentId, UtxoStore<Rindex<rindex::Address>>>,
}

impl Wallet {
    pub fn from_root_key(root_key: Key<XPrv, Rindex<rindex::Root>>) -> Self {
        let payload_key = root_key.hd_key();
        Self {
            root_key,
            payload_key,
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

    pub fn check_fragments<'a, I>(&mut self, fragments: I) -> bool
    where
        I: Iterator<Item = &'a Fragment>,
    {
        let mut at_least_once = false;

        for fragment in fragments {
            let fragment_id = fragment.hash();
            at_least_once |= self.check_fragment(&fragment_id, fragment);
        }

        at_least_once
    }

    pub fn check_fragment(&mut self, fragment_id: &FragmentId, fragment: &Fragment) -> bool {
        if self.state.contains(fragment_id) {
            return true;
        }

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

    pub fn check_address(&self, address: &OldAddress) -> bool {
        self.check(address).is_some()
    }

    pub(crate) fn check(&self, address: &OldAddress) -> Option<Key<XPrv, Rindex<rindex::Address>>> {
        let extended = address.deconstruct();
        let dp = self.derivation_path(&extended)?;

        let key_xprv = self.root_key.key(&dp);
        let key_xpub = key_xprv.public();

        if address.identical_with_xpub(key_xpub.public_key()) == AddressMatchXPub::Yes {
            Some(key_xprv)
        } else {
            None
        }
    }

    /// retrieve the derivation path from the extended address if possible
    ///
    /// if there is no derivation path, maybe this is a bip44 address
    /// if it is not possible to decrypt the payload it is not associated
    /// to this wallet
    fn derivation_path(
        &self,
        address: &ExtendedAddr,
    ) -> Option<DerivationPath<Rindex<rindex::Address>>> {
        let payload = address.attributes.derivation_path.as_deref()?;
        self.decode_payload(payload)
    }

    /// decode the payload expecting to retrieve the derivation path
    /// encrypted and encoded in cbor
    fn decode_payload(&self, payload: &[u8]) -> Option<DerivationPath<Rindex<rindex::Address>>> {
        let payload = self.payload_key.decrypt(payload).ok()?;

        decode_derivation_path(&payload)
            // assume derivation path will be RIndex. Even if this is not the case
            // and the decoded address is actually longer or shorter. Here we make
            // ourselves lenient to error
            .map(|dp| dp.coerce_unchecked())
    }
}
