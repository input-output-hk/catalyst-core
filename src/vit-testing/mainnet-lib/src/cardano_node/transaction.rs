use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::metadata::{AuxiliaryData, GeneralTransactionMetadata};
use cardano_serialization_lib::utils::{Coin, Value};
use cardano_serialization_lib::{
    Transaction, TransactionBody, TransactionInputs, TransactionOutput, TransactionOutputs,
    TransactionWitnessSet,
};

/// Transaction builder for cardano mainnet
pub struct TransactionBuilder;

impl TransactionBuilder {
    /// Builds transaction with metadata
    #[must_use]
    pub fn build_transaction_with_metadata(
        address: &Address,
        stake: u64,
        metadata: &GeneralTransactionMetadata,
    ) -> Transaction {
        let mut auxiliary_data = AuxiliaryData::new();
        auxiliary_data.set_metadata(metadata);

        let mut outputs = TransactionOutputs::new();
        outputs.add(&TransactionOutput::new(
            address,
            &Value::new(&Coin::from(stake)),
        ));

        let transaction_body =
            TransactionBody::new_tx_body(&TransactionInputs::new(), &outputs, &Coin::zero());
        Transaction::new(
            &transaction_body,
            &TransactionWitnessSet::new(),
            Some(auxiliary_data),
        )
    }
}
