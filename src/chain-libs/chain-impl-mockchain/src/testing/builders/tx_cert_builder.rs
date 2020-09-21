use crate::{
    certificate::{
        Certificate, CertificatePayload, PoolOwnersSigned, PoolSignature, TallyProof, VotePlan,
        VotePlanProof, VoteTally,
    },
    chaintypes::HeaderId,
    fee::FeeAlgorithm,
    fee::LinearFee,
    fragment::Fragment,
    key::EitherEd25519SecretKey,
    ledger::ledger::OutputAddress,
    testing::{data::Wallet, make_witness},
    transaction::{
        AccountBindingSignature, Input, Payload, SetAuthData, SetIOs,
        SingleAccountBindingSignature, TxBuilder, TxBuilderState, Witness,
    },
    value::Value,
    vote::PayloadType,
};

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
        builder: TxBuilderState<SetIOs<P>>,
        funder: &Wallet,
        inputs: &[Input],
        outputs: &[OutputAddress],
        should_make_witness: bool,
    ) -> TxBuilderState<SetAuthData<P>> {
        //utxo not supported yet
        let builder = builder.set_ios(inputs, outputs);

        let witnesses: Vec<Witness> = {
            if should_make_witness {
                let witness = make_witness(
                    self.block0_hash(),
                    &funder.as_account_data(),
                    &builder.get_auth_data_for_witness().hash(),
                );
                vec![witness]
            } else {
                vec![]
            }
        };
        builder.set_witnesses_unchecked(&witnesses)
    }

    fn fragment(
        &self,
        cert: &Certificate,
        keys: Vec<EitherEd25519SecretKey>,
        inputs: &[Input],
        outputs: &[OutputAddress],
        make_witness: bool,
        funder: &Wallet,
    ) -> Fragment {
        match cert {
            Certificate::StakeDelegation(s) => {
                let builder = self.set_initial_ios(
                    TxBuilder::new().set_payload(s),
                    &funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let signature =
                    AccountBindingSignature::new_single(&builder.get_auth_data(), |d| {
                        keys[0].sign_slice(&d.0)
                    });
                let tx = builder.set_payload_auth(&signature);
                Fragment::StakeDelegation(tx)
            }
            Certificate::PoolRegistration(s) => {
                let builder = self.set_initial_ios(
                    TxBuilder::new().set_payload(s),
                    &funder,
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
                    TxBuilder::new().set_payload(s),
                    &funder,
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
                    TxBuilder::new().set_payload(s),
                    &funder,
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
                    TxBuilder::new().set_payload(s),
                    &funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let tx = builder.set_payload_auth(&());
                Fragment::OwnerStakeDelegation(tx)
            }
            Certificate::VotePlan(vp) => {
                let builder = self.set_initial_ios(
                    TxBuilder::new().set_payload(vp),
                    &funder,
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
                    TxBuilder::new().set_payload(vp),
                    &funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let tx = builder.set_payload_auth(&());
                Fragment::VoteCast(tx)
            }
            Certificate::VoteTally(vt) => {
                let builder = self.set_initial_ios(
                    TxBuilder::new().set_payload(vt),
                    &funder,
                    inputs,
                    outputs,
                    make_witness,
                );
                let committee_signature = tally_sign(&keys, vt, &builder);
                let tx = builder.set_payload_auth(&committee_signature);
                Fragment::VoteTally(tx)
            }
        }
    }

    pub fn make_transaction(self, signers: &[&Wallet], certificate: &Certificate) -> Fragment {
        self.make_transaction_different_signers(&signers[0], signers, certificate)
    }

    pub fn make_transaction_different_signers(
        self,
        funder: &Wallet,
        signers: &[&Wallet],
        certificate: &Certificate,
    ) -> Fragment {
        let keys = signers
            .iter()
            .cloned()
            .map(|owner| owner.private_key())
            .collect();
        let input = funder.make_input_with_value(self.fee(certificate));
        self.fragment(certificate, keys, &[input], &[], true, funder)
    }
}

pub fn tally_sign(
    keys: &[EitherEd25519SecretKey],
    vt: &VoteTally,
    builder: &TxBuilderState<SetAuthData<VoteTally>>,
) -> TallyProof {
    let payload_type = vt.tally_type();

    match payload_type {
        PayloadType::Public => {
            let key: EitherEd25519SecretKey = keys[0].clone();
            let id = key.to_public().into();

            let auth_data = builder.get_auth_data();
            let signature =
                SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(&d.0));

            TallyProof::Public { id, signature }
        }
    }
}

pub fn plan_sign(
    keys: &[EitherEd25519SecretKey],
    builder: &TxBuilderState<SetAuthData<VotePlan>>,
) -> VotePlanProof {
    let key: EitherEd25519SecretKey = keys[0].clone();
    let id = key.to_public().into();

    let auth_data = builder.get_auth_data();
    let signature = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(&d.0));

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
        let sig = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(&d.0));
        sigs.push((i as u8, sig))
    }
    PoolOwnersSigned { signatures: sigs }
}

/// this struct can create any transaction including not valid one
/// in order to test robustness of ledger
pub struct FaultTolerantTxCertBuilder {
    builder: TestTxCertBuilder,
    cert: Certificate,
    funder: Wallet,
}

impl FaultTolerantTxCertBuilder {
    pub fn new(block0_hash: HeaderId, fee: LinearFee, cert: Certificate, funder: Wallet) -> Self {
        Self {
            builder: TestTxCertBuilder::new(block0_hash, fee),
            cert,
            funder,
        }
    }

    pub fn transaction_no_witness(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder
            .fragment(&self.cert, keys, &[input], &[], false, &self.funder)
    }

    pub fn transaction_input_to_low(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input_value = Value(1);
        let input = self.funder.make_input_with_value(input_value);
        let output = self.funder.make_output_with_value(Value(2));
        self.builder
            .fragment(&self.cert, keys, &[input], &[output], false, &self.funder)
    }

    pub fn transaction_with_input_output(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input_value = Value(self.builder.fee(&self.cert).0 + 1);
        let input = self.funder.make_input_with_value(input_value);
        let output = self.funder.make_output_with_value(Value(1));
        self.builder
            .fragment(&self.cert, keys, &[input], &[output], false, &self.funder)
    }

    pub fn transaction_with_output_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let output = self
            .funder
            .make_output_with_value(self.builder.fee(&self.cert));
        self.builder
            .fragment(&self.cert, keys, &[], &[output], false, &self.funder)
    }

    pub fn transaction_with_input_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder
            .fragment(&self.cert, keys, &[input], &[], false, &self.funder)
    }

    pub fn transaction_with_witness(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self
            .funder
            .make_input_with_value(self.builder.fee(&self.cert));
        self.builder
            .fragment(&self.cert, keys, &[input], &[], false, &self.funder)
    }
}
