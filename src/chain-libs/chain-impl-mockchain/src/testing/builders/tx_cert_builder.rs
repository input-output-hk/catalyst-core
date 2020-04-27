use crate::{
    certificate::{Certificate, CertificatePayload, PoolOwnersSigned, PoolSignature},
    chaintypes::HeaderId,
    fee::FeeAlgorithm,
    fee::LinearFee,
    fragment::Fragment,
    key::EitherEd25519SecretKey,
    testing::{builders::make_witness, data::Wallet},
    transaction::{
        AccountBindingSignature, Payload, SetAuthData, SetIOs, SingleAccountBindingSignature,
        TxBuilder, TxBuilderState,
    },
    value::Value,
};

pub struct TestTxCertBuilder {
    block0_hash: HeaderId,
    fee: LinearFee,
}

impl TestTxCertBuilder {
    pub fn new(block0_hash: HeaderId, fee: LinearFee) -> Self {
        Self { block0_hash, fee }
    }

    fn block0_hash(&self) -> &HeaderId {
        &self.block0_hash
    }

    fn fee(&self, certificate: &Certificate) -> Value {
        let payload: CertificatePayload = certificate.into();
        self.fee.calculate(Some(payload.as_slice()), 1, 0)
    }

    fn set_initial_ios<P: Payload>(
        &self,
        builder: TxBuilderState<SetIOs<P>>,
        funder: &Wallet,
        cert: &Certificate,
    ) -> TxBuilderState<SetAuthData<P>> {
        //utxo not supported yet
        let input = funder.make_input_with_value(self.fee(cert));
        let builder = builder.set_ios(&[input], &[]);
        let witness = make_witness(
            self.block0_hash(),
            &funder.as_account_data(),
            &builder.get_auth_data_for_witness().hash(),
        );
        builder.set_witnesses(&[witness])
    }

    fn fragment(
        &self,
        cert: &Certificate,
        keys: Vec<EitherEd25519SecretKey>,
        funder: &Wallet,
    ) -> Fragment {
        match cert {
            Certificate::StakeDelegation(s) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(s), &funder, cert);
                let signature =
                    AccountBindingSignature::new_single(&builder.get_auth_data(), |d| {
                        keys[0].sign_slice(&d.0)
                    });
                let tx = builder.set_payload_auth(&signature);
                Fragment::StakeDelegation(tx)
            }
            Certificate::PoolRegistration(s) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(s), &funder, cert);
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolRegistration(tx)
            }
            Certificate::PoolRetirement(s) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(s), &funder, cert);
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolRetirement(tx)
            }
            Certificate::PoolUpdate(s) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(s), &funder, cert);
                let signature = pool_owner_sign(&keys, &builder);
                let tx = builder.set_payload_auth(&signature);
                Fragment::PoolUpdate(tx)
            }
            Certificate::OwnerStakeDelegation(s) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(s), &funder, cert);
                let tx = builder.set_payload_auth(&());
                Fragment::OwnerStakeDelegation(tx)
            }
            Certificate::VotePlan(vp) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(vp), &funder, cert);
                let tx = builder.set_payload_auth(&());
                Fragment::VotePlan(tx)
            }
            Certificate::VoteCast(vp) => {
                let builder = self.set_initial_ios(TxBuilder::new().set_payload(vp), &funder, cert);
                let tx = builder.set_payload_auth(&());
                Fragment::VoteCast(tx)
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
        self.fragment(certificate, keys, funder)
    }
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
