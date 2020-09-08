use chain_crypto::{Ed25519, Ed25519Extended, SecretKey, Signature};
use chain_impl_mockchain::{
    accounting::account::SpendingCounter,
    block::HeaderId,
    transaction::{TransactionSignDataHash, Witness, WitnessUtxoData},
};
use ed25519_bip32::XPrv;

use hdkeygen::Key;

pub trait WitnessBuilder {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness;
}

pub struct OldUtxoWitnessBuilder<D>(pub Key<XPrv, D>);
pub struct UtxoWitnessBuilder<K>(pub K);
pub enum AccountWitnessBuilder {
    Ed25519(SecretKey<Ed25519>, SpendingCounter),
    Ed25519Extended(SecretKey<Ed25519Extended>, SpendingCounter),
}

impl<D> WitnessBuilder for OldUtxoWitnessBuilder<D> {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        let xprv = &self.0;
        let some_bytes = xprv.chain_code();

        let pk = xprv.public().pk();
        Witness::new_old_utxo(
            block0,
            sign_data_hash,
            |data| {
                let sig = Signature::from_binary(xprv.sign::<&[u8], _>(data.as_ref()).to_bytes())
                    .expect("cannot have invalid signature here");
                (pk, sig)
            },
            &some_bytes,
        )
    }
}

impl<D> WitnessBuilder for UtxoWitnessBuilder<Key<XPrv, D>> {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        let xprv = &self.0;
        Witness::new_utxo(block0, sign_data_hash, |data| {
            Signature::from_binary(xprv.sign::<WitnessUtxoData, &[u8]>(data.as_ref()).as_ref())
                .unwrap()
        })
    }
}

impl WitnessBuilder for UtxoWitnessBuilder<SecretKey<Ed25519Extended>> {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        let key = &self.0;
        Witness::new_utxo(block0, sign_data_hash, |data| {
            Signature::from_binary(key.sign(data).as_ref()).unwrap()
        })
    }
}

impl WitnessBuilder for AccountWitnessBuilder {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        match self {
            AccountWitnessBuilder::Ed25519(key, spending_counter) => {
                Witness::new_account(block0, sign_data_hash, *spending_counter, |data| {
                    key.sign(data)
                })
            }
            AccountWitnessBuilder::Ed25519Extended(key, spending_counter) => {
                Witness::new_account(block0, sign_data_hash, *spending_counter, |data| {
                    key.sign(data)
                })
            }
        }
    }
}
