use super::AverageValue;
use crate::{key::Hash, testing::data::AddressData, transaction::Output, utxo::Ledger};
use chain_addr::{Address, Discrimination};
use quickcheck::{Arbitrary, Gen};
use std::{collections::HashMap, iter};

#[derive(Debug, Clone)]
pub struct ArbitaryLedgerUtxo(pub Ledger<Address>);

impl Arbitrary for ArbitaryLedgerUtxo {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut ledger = Ledger::new();
        let size = usize::arbitrary(g) % 50 + 1;
        let arbitrary_utxos: HashMap<Hash, (u8, Output<Address>)> = iter::from_fn(|| {
            let outs = match u8::arbitrary(g) % 2 {
                0 => (
                    0u8,
                    AddressData::utxo(Discrimination::Test)
                        .make_output(AverageValue::arbitrary(g).into()),
                ),
                1 => (
                    0u8,
                    AddressData::delegation(Discrimination::Test)
                        .make_output(AverageValue::arbitrary(g).into()),
                ),
                _ => unreachable!(),
            };
            Some((Hash::arbitrary(g), outs))
        })
        .take(size)
        .collect();

        for (key, value) in arbitrary_utxos {
            ledger = ledger.add(&key, &[value]).unwrap();
        }
        ArbitaryLedgerUtxo(ledger)
    }
}
