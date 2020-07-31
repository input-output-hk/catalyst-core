mod builder;
mod strategy;
mod witness_builder;

pub use self::witness_builder::{
    AccountWitnessBuilder, OldUtxoWitnessBuilder, UtxoWitnessBuilder, WitnessBuilder,
};
pub use self::{
    builder::{AddInputStatus, TransactionBuilder},
    strategy::{InputStrategy, OutputStrategy, Strategy, StrategyBuilder, DEFAULT_STRATEGIES},
};

use crate::store::UtxoStore;

use chain_impl_mockchain::transaction::{Balance, Input, NoExtra, Output, Transaction};

pub fn send_to_one_address<K: 'static>(
    settings: &crate::Settings,
    address: &chain_addr::Address,
    utxo_store: &UtxoStore<K>,
) -> Option<(Transaction<NoExtra>, Vec<Input>)> {
    let payload = chain_impl_mockchain::transaction::NoExtra;

    let mut builder = TransactionBuilder::new(settings, payload);
    let utxos = utxo_store.utxos();

    let mut ignored = vec![];

    // TODO: return this?
    let _new_store = utxos
        .take_while(|utxo| {
            let input = Input::from_utxo(*utxo.as_ref());

            let xprv = utxo_store.get_signing_key(utxo).unwrap();
            let witness_builder = OldUtxoWitnessBuilder(xprv.as_ref().clone());

            match builder.add_input(input, witness_builder) {
                AddInputStatus::Added => true,
                AddInputStatus::Skipped(input) => {
                    ignored.push(input);
                    true
                }
                AddInputStatus::NotEnoughSpace => false,
            }
        })
        .fold(utxo_store.clone(), |store, utxo| {
            store.remove(utxo).unwrap()
        });

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
