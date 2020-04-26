use crate::{
    chaintypes::HeaderId, testing::data::AddressData, transaction::TransactionSignDataHash,
};
use chain_addr::Kind;

pub use crate::transaction::Witness;

pub fn make_witnesses(
    block0: &HeaderId,
    addresses_data: Vec<&AddressData>,
    transaction_hash: &TransactionSignDataHash,
) -> Vec<Witness> {
    addresses_data
        .iter()
        .map(|x| make_witness(block0, x, transaction_hash))
        .collect()
}

pub fn make_witness(
    block0: &HeaderId,
    addres_data: &AddressData,
    transaction_hash: &TransactionSignDataHash,
) -> Witness {
    match addres_data.address.kind() {
        Kind::Account(_) => Witness::new_account(
            block0,
            transaction_hash,
            addres_data.spending_counter.unwrap(),
            |d| addres_data.private_key().sign(d),
        ),
        _ => Witness::new_utxo(block0, transaction_hash, |d| {
            addres_data.private_key().sign(d)
        }),
    }
}
