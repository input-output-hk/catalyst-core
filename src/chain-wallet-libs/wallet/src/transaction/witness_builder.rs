use chain_crypto::{Ed25519, PublicKey, SecretKey, Signature};
use chain_impl_mockchain::{
    chaintypes::HeaderId,
    transaction::{TransactionSignDataHash, Witness},
};
use ed25519_bip32::XPrv;
use hdkeygen::account::Account;

pub(crate) enum WitnessBuilder {
    OldUtxo { xprv: XPrv },
    Account { account: Account },
}

impl WitnessBuilder {
    pub(crate) fn mk_witness(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> Witness {
        match self {
            Self::OldUtxo { xprv } => {
                let some_bytes = xprv.chain_code();
                let pk = PublicKey::from_binary(&xprv.public().public_key())
                    .expect("cannot have an invalid public key here");
                Witness::new_old_utxo(
                    block0,
                    sign_data_hash,
                    |data| {
                        let sig = Signature::from_binary(xprv.sign::<()>(data.as_ref()).to_bytes())
                            .expect("cannot have invalid signature here");
                        (pk, sig)
                    },
                    &some_bytes,
                )
            }
            Self::Account { account } => {
                let key = account.seed();
                let spending_counter = account.counter().into();
                let key = SecretKey::<Ed25519>::from_binary(key)
                    .expect("an account key should already be the right size and format");

                Witness::new_account(block0, sign_data_hash, &spending_counter, |data| {
                    key.sign(data)
                })
            }
        }
    }
}
