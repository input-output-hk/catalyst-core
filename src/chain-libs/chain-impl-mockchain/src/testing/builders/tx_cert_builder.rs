use crate::{
    certificate::{
        BftLeaderBindingSignature, Certificate, CertificatePayload, EvmMapping, PoolOwnersSigned,
        PoolSignature, TallyProof, UpdateProposal, UpdateVote, VotePlan, VotePlanProof, VoteTally,
    },
    chaintypes::HeaderId,
    date::BlockDate,
    fee::FeeAlgorithm,
    fee::LinearFee,
    fragment::Fragment,
    key::EitherEd25519SecretKey,
    ledger::ledger::OutputAddress,
    testing::{data::Wallet, make_witness, make_witness_with_lane},
    transaction::{
        AccountBindingSignature, Input, Payload, SetAuthData, SetTtl,
        SingleAccountBindingSignature, TxBuilder, TxBuilderState, Witness,
    },
    value::Value,
    vote::PayloadType,
};

use std::iter;

#[derive(Debug, Copy, Clone)]
pub enum WitnessMode {
    None,
    Default,
    Account { lane: usize },
}

impl Default for WitnessMode {
    fn default() -> Self {
        Self::Default
    }
}

pub struct TestTxCertBuilder {
    block0_hash: HeaderId,
    fee: LinearFee,
}

impl TestTxCertBuilder {
    pub fn new(block0_hash: HeaderId, fee: LinearFee) -> Self {
        Self { block0_hash, fee }
    }

    pub fn block0_hash(&self) -> &HeaderId {
        &self.block0_hash
    }

    pub fn fee(&self, certificate: &Certificate) -> Value {
        let payload: CertificatePayload = certificate.into();
        self.fee.calculate(Some(payload.as_slice()), 1, 0)
    }

    fn set_initial_ios<P: Payload>(
        &self,
        valid_until: BlockDate,
        builder: TxBuilderState<SetTtl<P>>,
        funder: &Wallet,
        inputs: &[Input],
        outputs: &[OutputAddress],
        witness_mode: WitnessMode,
    ) -> TxBuilderState<SetAuthData<P>> {
        //utxo not supported yet
        let builder = builder
            .set_expiry_date(valid_until)
            .set_ios(inputs, outputs);

        let witnesses: Vec<Witness> = {
            match witness_mode {
                WitnessMode::None => vec![],
                WitnessMode::Default => vec![make_witness(
                    self.block0_hash(),
                    &funder.as_account_data(),
                    &builder.get_auth_data_for_witness().hash(),
                )],
                WitnessMode::Account { lane } => vec![make_witness_with_lane(
                    self.block0_hash(),
                    &funder.as_account_data(),
                    lane,
                    &builder.get_auth_data_for_witness().hash(),
                )],
            }
        };
        builder.set_witnesses_unchecked(&witnesses)
    }

    // A builder API would be better, but it's an internal function.
    #[allow(clippy::too_many_arguments)]
    fn fragment(
        &self,
        valid_until: BlockDate,
        cert: &Certificate,
        keys: Vec<EitherEd25519SecretKey>,
        inputs: &[Input],
        outputs: &[OutputAddress],
        make_witness: WitnessMode,
        funder: &Wallet,
    ) -> Fragment {
        match cert {
            Certificate::StakeDelegation(s) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(s),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature =
                    AccountBindingSignature::new_single(&builder.get_auth_data(), |d| {
                        keys[0].sign_slice(d.0)
                    });
                let tx = builder.set_payload_auth(&signature);
                Fragment::StakeDelegation(tx)
            }
            Certificate::PoolRegistration(s) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(s),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolRegistration(tx)
            }
            Certificate::PoolRetirement(s) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(s),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolRetirement(tx)
            }
            Certificate::PoolUpdate(s) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(s),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolUpdate(tx)
            }
            Certificate::OwnerStakeDelegation(s) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(s),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let tx = builder.set_payload_auth(&());
                Fragment::OwnerStakeDelegation(tx)
            }
            Certificate::VotePlan(vp) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(vp),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let committee_signature = plan_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&committee_signature);
                Fragment::VotePlan(tx)
            }
            Certificate::VoteCast(vp) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(vp),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let tx = builder.set_payload_auth(&());
                Fragment::VoteCast(tx)
            }
            Certificate::VoteTally(vt) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(vt),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let committee_signature = tally_sign(&keys, vt, &builder);
                let tx = builder.set_payload_auth(&committee_signature);
                Fragment::VoteTally(tx)
            }
            Certificate::UpdateProposal(update_proposal) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(update_proposal),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = update_proposal_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::UpdateProposal(tx)
            }
            Certificate::UpdateVote(update_vote) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(update_vote),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = update_vote_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::UpdateVote(tx)
            }
            Certificate::MintToken(mint_token) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(mint_token),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let tx = builder.set_payload_auth(&());
                Fragment::MintToken(tx)
            }
            Certificate::EvmMapping(evm_mapping) => {
                let builder = self.set_initial_ios(
                    valid_until,
                    TxBuilder::new().set_payload(evm_mapping),
                    funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature = evm_mapping_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::EvmMapping(tx)
            }
        }
    }

    pub fn make_transaction<'a, T>(
        self,
        valid_until: BlockDate,
        signers: T,
        certificate: &Certificate,
        witness_mode: WitnessMode,
    ) -> Fragment
    where
        T: IntoIterator<Item = &'a Wallet>,
    {
        let mut remainder = signers.into_iter();
        let funder = remainder.next().expect("needs at least one signer");
        self.make_transaction_different_signers(
            valid_until,
            funder,
            iter::once(funder).chain(remainder),
            certificate,
            witness_mode,
        )
    }

    pub fn make_transaction_different_signers<'a, T>(
        self,
        valid_until: BlockDate,
        funder: &'a Wallet,
        signers: T,
        certificate: &Certificate,
        witness_mode: WitnessMode,
    ) -> Fragment
    where
        T: IntoIterator<Item = &'a Wallet>,
    {
        let keys = signers.into_iter().map(|x| x.private_key()).collect();
        let input = funder.make_input_with_value(self.fee(certificate));
        self.fragment(
            valid_until,
            certificate,
            keys,
            &[input],
            &[],
            witness_mode,
            funder,
        )
    }
}

pub fn tally_sign(
    keys: &[EitherEd25519SecretKey],
    vt: &VoteTally,
    builder: &TxBuilderState<SetAuthData<VoteTally>>,
) -> TallyProof {
    let payload_type = vt.tally_type();

    let key: EitherEd25519SecretKey = keys[0].clone();
    let id = key.to_public().into();

    let auth_data = builder.get_auth_data();
    let signature = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(d.0));

    match payload_type {
        PayloadType::Public => TallyProof::Public { id, signature },
        PayloadType::Private => TallyProof::Private { id, signature },
    }
}

pub fn plan_sign(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<VotePlan>>,
) -> VotePlanProof {
    let key: EitherEd25519SecretKey = keys[0].clone();
    let id = key.to_public().into();

    let auth_data = builder.get_auth_data();
    let signature = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(d.0));

    VotePlanProof { id, signature }
}

pub fn pool_owner_sign<P: Payload>(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<P>>,
) -> PoolSignature {
    let pool_owner = pool_owner_signed(keys, builder);
    PoolSignature::Owners(pool_owner)
}

pub fn pool_owner_signed<P: Payload>(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<P>>,
) -> PoolOwnersSigned {
    let auth_data = builder.get_auth_data();
    let mut sigs = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let sig = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(d.0));
        sigs.push((i as u8, sig))
    }
    PoolOwnersSigned { signatures: sigs }
}

pub fn update_proposal_sign(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<UpdateProposal>>,
) -> BftLeaderBindingSignature {
    let key: EitherEd25519SecretKey = keys[0].clone();

    let auth_data = builder.get_auth_data();
    BftLeaderBindingSignature::new(&auth_data, |d| key.sign_slice(d.0))
}

pub fn update_vote_sign(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<UpdateVote>>,
) -> BftLeaderBindingSignature {
    let key: EitherEd25519SecretKey = keys[0].clone();

    let auth_data = builder.get_auth_data();
    BftLeaderBindingSignature::new(&auth_data, |d| key.sign_slice(d.0))
}

pub fn evm_mapping_sign(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<EvmMapping>>,
) -> SingleAccountBindingSignature {
    let key: EitherEd25519SecretKey = keys[0].clone();

    let auth_data = builder.get_auth_data();
    SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(d.0))
}

/// this struct can create any transaction including not valid one
/// in order to test robustness of ledger
pub struct FaultTolerantTxCertBuilder {
    builder: TestTxCertBuilder,
    valid_until: BlockDate,
    cert: Certificate,
    funder: Wallet,
}

impl FaultTolerantTxCertBuilder {
    pub fn new(
        block0_hash: HeaderId,
        fee: LinearFee,
        cert: Certificate,
        valid_until: BlockDate,
        funder: Wallet,
    ) -> Self {
        Self {
            builder: TestTxCertBuilder::new(block0_hash, fee),
            cert,
            funder,
            valid_until,
        }
    }

    pub fn transaction_no_witness(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[input],
            &[],
            Default::default(),
            &self.funder,
        )
    }

    pub fn transaction_input_to_low(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input_value = Value(1);
        let input = self.funder.make_input_with_value(input_value);
        let output = self.funder.make_output_with_value(Value(2));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[input],
            &[output],
            Default::default(),
            &self.funder,
        )
    }

    pub fn transaction_with_input_output(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input_value = Value(self.builder.fee(&self.cert).0 + 1);
        let input = self.funder.make_input_with_value(input_value);
        let output = self.funder.make_output_with_value(Value(1));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[input],
            &[output],
            Default::default(),
            &self.funder,
        )
    }

    pub fn transaction_with_output_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let output = self
            .funder
            .make_output_with_value(self.builder.fee(&self.cert));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[],
            &[output],
            Default::default(),
            &self.funder,
        )
    }

    pub fn transaction_with_input_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[input],
            &[],
            Default::default(),
            &self.funder,
        )
    }

    pub fn transaction_with_witness(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder.fragment(
            self.valid_until,
            &self.cert,
            keys,
            &[input],
            &[],
            Default::default(),
            &self.funder,
        )
    }
}
