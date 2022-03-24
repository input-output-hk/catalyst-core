use crate::store::{Groupable, UtxoStore};

use super::builder::{AddInputStatus, TransactionBuilder};
use super::witness_builder::{UtxoWitnessBuilder, WitnessBuilder};
use chain_impl_mockchain::{
    block::BlockDate,
    fragment::Fragment,
    transaction::{Balance, Input, NoExtra, Output, Transaction},
};

pub struct DumpIter<'a, W> {
    settings: &'a crate::Settings,
    address: &'a chain_addr::Address,
    wallet: &'a mut W,
    valid_until: BlockDate,
}

pub type DumpFreeKeys<'a> = DumpIter<'a, crate::scheme::freeutxo::Wallet>;

pub fn send_to_one_address<K: Clone + Groupable, WB: WitnessBuilder>(
    settings: &crate::Settings,
    address: &chain_addr::Address,
    utxo_store: &UtxoStore<K>,
    mk_witness: &'static dyn Fn(K) -> WB,
    valid_until: BlockDate,
) -> Option<(Transaction<NoExtra>, Vec<Input>)> {
    let payload = chain_impl_mockchain::transaction::NoExtra;

    let mut builder = TransactionBuilder::new(settings, payload, valid_until);
    let utxos = utxo_store.utxos();

    let mut ignored = vec![];

    for utxo in utxos {
        let input = Input::from_utxo(*utxo.as_ref());

        let key = utxo_store.get_signing_key(utxo).unwrap();
        let witness_builder = mk_witness((*key).clone());

        match builder.add_input_if_worth(input, witness_builder) {
            AddInputStatus::Added => (),
            AddInputStatus::Skipped(input) => {
                ignored.push(input);
            }
            AddInputStatus::NotEnoughSpace => break,
        }
    }

    if builder.inputs().is_empty() {
        None
    } else {
        match builder.check_balance_with(0, 1) {
            Balance::Positive(value) => {
                builder.add_output(Output::from_address(address.clone(), value));
            }
            Balance::Zero => (),
            Balance::Negative(_) => unreachable!(),
        }

        let fragment = builder.finalize_tx(()).unwrap();

        Some((fragment, ignored))
    }
}

pub fn dump_free_utxo<'a>(
    settings: &'a crate::Settings,
    address: &'a chain_addr::Address,
    wallet: &'a mut crate::scheme::freeutxo::Wallet,
    valid_until: BlockDate,
) -> DumpFreeKeys<'a> {
    DumpFreeKeys {
        settings,
        address,
        wallet,
        valid_until,
    }
}

impl<'a> Iterator for DumpFreeKeys<'a> {
    type Item = (Fragment, Vec<Input>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = send_to_one_address(
            self.settings,
            self.address,
            self.wallet.utxos(),
            &UtxoWitnessBuilder,
            self.valid_until,
        )
        .map(|(tx, ignored)| (Fragment::Transaction(tx), ignored));

        if let Some((fragment, _)) = next.as_ref() {
            self.wallet.check_fragment(&fragment.hash(), fragment);
        }

        next
    }
}
