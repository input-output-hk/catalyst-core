use chain_crypto::{Ed25519, SecretKey, Signature};
use chain_impl_mockchain::{
    block::HeaderId,
    transaction::{TransactionSignDataHash, Witness},
};
use ed25519_bip32::XPrv;
use hdkeygen::account::Account;
use hdkeygen::Key;

pub trait WitnessBuilder {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness;
}

pub struct OldUtxoWitnessBuilder<D>(pub Key<XPrv, D>);
pub struct UtxoWitnessBuilder<D>(pub Key<XPrv, D>);
pub struct AccountWitnessBuilder(pub Account);

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

impl<D> WitnessBuilder for UtxoWitnessBuilder<D> {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        let xprv = &self.0;
        Witness::new_utxo(block0, sign_data_hash, |data| {
            Signature::from_binary(xprv.sign::<&[u8], _>(data.as_ref()).to_bytes())
                .expect("cannot have invalid signature here")
        })
    }
}

impl WitnessBuilder for AccountWitnessBuilder {
    fn build(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        let account = &self.0;
        let key = account.seed();
        let spending_counter = account.counter().into();
        let key = SecretKey::<Ed25519>::from_binary(key)
            .expect("an account key should already be the right size and format");

        Witness::new_account(block0, sign_data_hash, spending_counter, |data| {
            key.sign(data)
        })
    }
}
