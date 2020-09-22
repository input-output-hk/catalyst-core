use crate::{
    certificate::{Certificate, PoolUpdate, VoteCast, VotePlan, VoteTally},
    fragment::Fragment,
    key::EitherEd25519SecretKey,
    ledger::ledger::OutputAddress,
    testing::{
        builders::*,
        data::{StakePool, Wallet},
    },
    transaction::*,
    value::Value,
};
use std::vec::Vec;

///Below method should be used only for negative testing
pub fn create_initial_stake_pool_update(
    stake_pool_update: &PoolUpdate,
    owners: &[Wallet],
) -> Fragment {
    let cert = build_stake_pool_update_cert(&stake_pool_update);
    let keys: Vec<EitherEd25519SecretKey> = owners
        .iter()
        .cloned()
        .map(|owner| owner.private_key())
        .collect();
    fragment(cert, keys, &[], &[])
}

pub fn create_initial_stake_pool_registration(
    stake_pool: &StakePool,
    owners: &[Wallet],
) -> Fragment {
    let cert = build_stake_pool_registration_cert(&stake_pool.info());
    let keys: Vec<EitherEd25519SecretKey> = owners
        .iter()
        .cloned()
        .map(|owner| owner.private_key())
        .collect();
    fragment(cert, keys, &[], &[])
}

pub fn create_initial_vote_plan(vote_plan: &VotePlan, owners: &[Wallet]) -> Fragment {
    let cert: Certificate = vote_plan.clone().into();
    let keys: Vec<EitherEd25519SecretKey> = owners
        .iter()
        .cloned()
        .map(|owner| owner.private_key())
        .collect();
    fragment(cert, keys, &[], &[])
}

pub fn create_initial_vote_cast(vote_cast: &VoteCast, owners: &[Wallet]) -> Fragment {
    let cert: Certificate = vote_cast.clone().into();
    let keys: Vec<EitherEd25519SecretKey> = owners
        .iter()
        .cloned()
        .map(|owner| owner.private_key())
        .collect();
    fragment(cert, keys, &[], &[])
}

pub fn create_initial_vote_tally(vote_tally: &VoteTally, owners: &[Wallet]) -> Fragment {
    let cert: Certificate = vote_tally.clone().into();
    let keys: Vec<EitherEd25519SecretKey> = owners
        .iter()
        .cloned()
        .map(|owner| owner.private_key())
        .collect();
    fragment(cert, keys, &[], &[])
}

pub fn create_initial_transaction(wallet: &Wallet) -> Fragment {
    let tx = TxBuilder::new()
        .set_nopayload()
        .set_ios(&[], &[wallet.make_output()])
        .set_witnesses(&[])
        .set_payload_auth(&());
    Fragment::Transaction(tx)
}

pub fn create_initial_stake_pool_delegation(stake_pool: &StakePool, wallet: &Wallet) -> Fragment {
    let cert = build_stake_delegation_cert(&stake_pool.info(), &wallet.as_account_data());
    let keys: Vec<EitherEd25519SecretKey> = vec![wallet.private_key()];
    fragment(cert, keys, &[], &[])
}

fn set_initial_ios<P: Payload>(
    builder: TxBuilderState<SetIOs<P>>,
    inputs: &[Input],
    outputs: &[OutputAddress],
) -> TxBuilderState<SetAuthData<P>> {
    builder.set_ios(inputs, outputs).set_witnesses(&[])
}

fn fragment(
    cert: Certificate,
    keys: Vec<EitherEd25519SecretKey>,
    inputs: &[Input],
    outputs: &[OutputAddress],
) -> Fragment {
    match cert {
        Certificate::StakeDelegation(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let signature = AccountBindingSignature::new_single(&builder.get_auth_data(), |d| {
                keys[0].sign_slice(&d.0)
            });
            let tx = builder.set_payload_auth(&signature);
            Fragment::StakeDelegation(tx)
        }
        Certificate::PoolRegistration(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let signature = pool_owner_sign(&keys, &builder);
            let tx = builder.set_payload_auth(&signature);
            Fragment::PoolRegistration(tx)
        }
        Certificate::PoolUpdate(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let signature = pool_owner_sign(&keys, &builder);
            let tx = builder.set_payload_auth(&signature);
            Fragment::PoolUpdate(tx)
        }
        Certificate::VotePlan(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let signature = plan_sign(&keys, &builder);
            let tx = builder.set_payload_auth(&signature);
            Fragment::VotePlan(tx)
        }
        Certificate::VoteCast(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let tx = builder.set_payload_auth(&());
            Fragment::VoteCast(tx)
        }
        Certificate::VoteTally(s) => {
            let builder = set_initial_ios(TxBuilder::new().set_payload(&s), inputs, outputs);
            let signature = tally_sign(&keys,&s, &builder);
            let tx = builder.set_payload_auth(&signature);
            Fragment::VoteTally(tx)
        }
        _ => unreachable!(),
    }
}

pub struct InitialFaultTolerantTxCertBuilder {
    cert: Certificate,
    funder: Wallet,
}

impl InitialFaultTolerantTxCertBuilder {
    pub fn new(cert: Certificate, funder: Wallet) -> Self {
        Self { cert, funder }
    }

    pub fn transaction_with_input_output(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self.funder.make_input_with_value(Value(1));
        let output = self.funder.make_output_with_value(Value(1));
        fragment(self.cert.clone(), keys, &[input], &[output])
    }

    pub fn transaction_with_output_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let output = self.funder.make_output_with_value(Value(1));
        fragment(self.cert.clone(), keys, &[], &[output])
    }

    pub fn transaction_with_input_only(&self) -> Fragment {
        let keys = vec![self.funder.private_key()];
        let input = self.funder.make_input_with_value(Value(1));
        fragment(self.cert.clone(), keys, &[input], &[])
    }
}
