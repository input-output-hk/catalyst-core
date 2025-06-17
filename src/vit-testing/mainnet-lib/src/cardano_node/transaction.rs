use cardano_serialization_lib::{
    Address, AuxiliaryData, Coin, GeneralTransactionMetadata, Transaction, TransactionBody,
    TransactionInputs, TransactionOutput, TransactionOutputs, TransactionWitnessSet, Value,
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
