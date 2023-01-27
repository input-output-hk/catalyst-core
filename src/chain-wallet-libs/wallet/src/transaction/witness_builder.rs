use chain_crypto::{Ed25519, Ed25519Extended, SecretKey, Signature};
use chain_impl_mockchain::{
    accounting::account::SpendingCounter,
    block::HeaderId,
    transaction::{TransactionSignDataHash, Witness, WitnessAccountData, WitnessUtxoData},
};
use ed25519_bip32::XPrv;

use hdkeygen::Key;

pub trait WitnessBuilder<SecretKey, WitnessData: AsRef<[u8]>> {
    fn build_sign_data(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessData;

    fn sign(&self, witness_data: WitnessData, secret_key: SecretKey) -> Witness;

    fn build(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
        secret_key: SecretKey,
    ) -> Witness {
        self.sign(self.build_sign_data(block0, sign_data_hash), secret_key)
    }
}

pub struct UtxoWitnessBuilder;

pub enum AccountSecretKey {
    Ed25519(SecretKey<Ed25519>),
    Ed25519Extended(SecretKey<Ed25519Extended>),
}

pub struct AccountWitnessBuilder(pub SpendingCounter);

impl<D> WitnessBuilder<Key<XPrv, D>, WitnessUtxoData> for UtxoWitnessBuilder {
    fn build_sign_data(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessUtxoData {
        Witness::new_utxo_data(block0, sign_data_hash)
    }

    fn sign(&self, witness_data: WitnessUtxoData, secret_key: Key<XPrv, D>) -> Witness {
        Witness::Utxo(
            Signature::from_binary(
                secret_key
                    .sign::<WitnessUtxoData, &[u8]>(witness_data.as_ref())
                    .as_ref(),
            )
            .unwrap(),
        )
    }
}

impl WitnessBuilder<SecretKey<Ed25519Extended>, WitnessUtxoData> for UtxoWitnessBuilder {
    fn build_sign_data(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessUtxoData {
        Witness::new_utxo_data(block0, sign_data_hash)
    }

    fn sign(
        &self,
        witness_data: WitnessUtxoData,
        secret_key: SecretKey<Ed25519Extended>,
    ) -> Witness {
        Witness::Utxo(secret_key.sign(&witness_data))
    }
}

impl WitnessBuilder<AccountSecretKey, WitnessAccountData> for AccountWitnessBuilder {
    fn build_sign_data(
        &self,
        block0: &HeaderId,
        sign_data_hash: &TransactionSignDataHash,
    ) -> WitnessAccountData {
        Witness::new_account_data(block0, sign_data_hash, self.0)
    }

    fn sign(&self, witness_data: WitnessAccountData, secret_key: AccountSecretKey) -> Witness {
        match secret_key {
            AccountSecretKey::Ed25519(secret_key) => {
                Witness::Account(self.0, secret_key.sign(&witness_data))
            }
            AccountSecretKey::Ed25519Extended(secret_key) => {
                Witness::Account(self.0, secret_key.sign(&witness_data))
            }
        }
    }
}
