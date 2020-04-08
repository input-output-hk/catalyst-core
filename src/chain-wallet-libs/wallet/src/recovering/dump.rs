use crate::Settings;
use chain_addr::Address;
use chain_crypto::{PublicKey, Signature};
use chain_impl_mockchain::{
    chaintypes::HeaderId,
    fee::FeeAlgorithm as _,
    transaction::{
        Input, NoExtra, Output, Transaction, TransactionSignDataHash, TxBuilderState, Witness,
    },
    value::Value,
};
use chain_path_derivation::rindex::{self, Rindex};
use ed25519_bip32::XPrv;
use hdkeygen::Key;

pub(crate) enum WitnessBuilder {
    OldUtxo {
        xprv: Key<XPrv, Rindex<rindex::Address>>,
    },
}

/// Dump all the values into transactions to fill one address
pub struct Dump {
    settings: Settings,
    address: Address,
    inputs: Vec<Input>,
    witness_builders: Vec<WitnessBuilder>,
    outputs: Vec<Transaction<NoExtra>>,
    ignored: Vec<Input>,
}

impl WitnessBuilder {
    fn mk_witness(&self, block0: &HeaderId, sign_data_hash: &TransactionSignDataHash) -> Witness {
        match self {
            Self::OldUtxo { xprv } => {
                let some_bytes = xprv.as_ref().chain_code();
                let pk = PublicKey::from_binary(&xprv.as_ref().public().public_key())
                    .expect("cannot have an invalid public key here");
                Witness::new_old_utxo(
                    block0,
                    sign_data_hash,
                    |data| {
                        let sig = Signature::from_binary(xprv.sign::<(), _>(data).to_bytes())
                            .expect("cannot have invalid signature here");
                        (pk, sig)
                    },
                    &some_bytes,
                )
            }
        }
    }
}

impl Dump {
    pub fn new(settings: Settings, address: Address) -> Self {
        Self {
            settings,
            address,
            inputs: Vec::with_capacity(255),
            witness_builders: Vec::with_capacity(255),
            outputs: Vec::new(),
            ignored: Vec::new(),
        }
    }

    fn inputs_value(&self) -> Value {
        self.inputs.iter().map(|i| i.value()).sum()
    }

    fn is_input_worth(&self, input: &Input) -> bool {
        let value = input.value();
        let minimal_value = self.settings.parameters.fees.fees_for_inputs_outputs(1, 0);

        value > minimal_value
    }

    fn estimate_fee(&self) -> Value {
        self.settings
            .parameters
            .fees
            .calculate(None, self.inputs.len() as u8, 1u8)
    }

    fn mk_output(&self) -> Option<Output<Address>> {
        let fee = self.estimate_fee();
        let input = self.inputs_value();

        let value = input.checked_sub(fee).ok()?;
        let address = self.address.clone();
        Some(Output::from_address(address, value))
    }

    fn build_tx_and_clear(&mut self) {
        if let Some(output) = self.mk_output() {
            let builder = TxBuilderState::new();
            let builder = builder.set_nopayload();
            let builder = builder.set_ios(&self.inputs, &[output]);

            let header_id = self.settings.static_parameters.block0_initial_hash;
            let auth_data = builder.get_auth_data_for_witness().hash();
            let witnesses = std::mem::replace(&mut self.witness_builders, Vec::with_capacity(255));
            let witnesses: Vec<_> = witnesses
                .into_iter()
                .map(move |wb| wb.mk_witness(&header_id, &auth_data))
                .collect();

            let builder = builder.set_witnesses(&witnesses);
            let tx = builder.set_payload_auth(&());
            self.outputs.push(tx);
        }

        self.witness_builders.clear();
        self.inputs.clear();
    }

    pub(crate) fn push(&mut self, input: Input, witness_builder: WitnessBuilder) {
        if self.inputs.len() >= 255 {
            self.build_tx_and_clear()
        }

        if self.is_input_worth(&input) {
            self.inputs.push(input);
            self.witness_builders.push(witness_builder);
        } else {
            self.ignored.push(input);
        }
    }
}
