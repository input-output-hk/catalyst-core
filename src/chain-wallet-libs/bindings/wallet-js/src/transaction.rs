use crate::certificates::Certificate;
use chain_impl_mockchain::{
    block::BlockDate as BlockDateLib,
    certificate::Certificate as CertificateLib,
    transaction::{
        Input as InputLib, Output as OutputLib, SetTtl,
        TransactionSignDataHash as TransactionSignDataHashLib, TxBuilder, TxBuilderState,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TransactionSignDataHash(pub(crate) TransactionSignDataHashLib);

#[wasm_bindgen]
pub struct BlockDate(pub(crate) BlockDateLib);

#[wasm_bindgen]
pub struct Input(pub(crate) InputLib);

#[wasm_bindgen]
pub struct Output(pub(crate) OutputLib<chain_addr::Address>);

#[wasm_bindgen]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    certificate: Option<Certificate>,
}

impl Transaction {
    pub fn build_transaction(
        certificate: Option<Certificate>,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
    ) -> Self {
        Self {
            certificate,
            inputs,
            outputs,
        }
    }

    fn sign_data_hash_impl<P>(
        &self,
        valid_until: BlockDate,
        builder: TxBuilderState<SetTtl<P>>,
    ) -> TransactionSignDataHash {
        let inputs: Vec<_> = self.inputs.iter().map(|input| input.0.clone()).collect();
        let outputs: Vec<_> = self.outputs.iter().map(|output| output.0.clone()).collect();
        TransactionSignDataHash(
            builder
                .set_expiry_date(valid_until.0)
                .set_ios(&inputs, &outputs)
                .get_auth_data_for_witness()
                .hash(),
        )
    }

    pub fn sign_data_hash(&self, valid_until: BlockDate) -> TransactionSignDataHash {
        if let Some(certificate) = &self.certificate {
            match &certificate.0 {
                CertificateLib::StakeDelegation(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::OwnerStakeDelegation(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::PoolRegistration(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::PoolRetirement(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::PoolUpdate(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::VotePlan(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::VoteCast(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::VoteTally(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::UpdateProposal(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::UpdateVote(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::MintToken(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
                CertificateLib::EvmMapping(p) => {
                    self.sign_data_hash_impl(valid_until, TxBuilder::new().set_payload(p))
                }
            }
        } else {
            self.sign_data_hash_impl(valid_until, TxBuilder::new().set_nopayload())
        }
    }
}
