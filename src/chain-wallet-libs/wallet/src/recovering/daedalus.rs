use super::RecoveryError;
use crate::{
    transaction::{Dump, WitnessBuilder},
    scheme::rindex::Wallet,
};
use chain_impl_mockchain::{
    fragment::Fragment,
    legacy::OldAddress,
    transaction::{Input, UtxoPointer},
    value::Value,
};
use chain_path_derivation::rindex::{self, Rindex};
use ed25519_bip32::XPrv;
use hdkeygen::{
    Key,
};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct RecoveringDaedalus {
    wallet: Wallet,
}

impl RecoveringDaedalus {
    pub(crate) fn new(wallet: Wallet) -> Self {
        Self {
            wallet,
        }
    }

    pub fn remove(&mut self, pointer: UtxoPointer) {
        if self.utxos.remove(&pointer).is_some() {
            self.value_total = self
                .value_total
                .checked_sub(pointer.value)
                .unwrap_or_else(|_| Value::zero())
        }
    }

    pub fn value_total(&self) -> Value {
        self.wallet.un
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
