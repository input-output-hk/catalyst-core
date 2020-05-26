use super::{
    element::SingleAccountBindingSignature, AccountBindingSignature, AccountIdentifier, Input,
    NoExtra, Payload, Transaction, TxBuilder, UnspecifiedAccountIdentifier, UtxoPointer, Witness,
};
#[cfg(test)]
use crate::certificate::OwnerStakeDelegation;
use crate::key::{EitherEd25519SecretKey, SpendingSignature};
use chain_crypto::{testing::arbitrary_secret_key, Ed25519, SecretKey, Signature};
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

quickcheck! {
    fn transaction_encode_decode(transaction: Transaction<NoExtra>) -> TestResult {
        chain_test_utils::property::serialization_bijection_r(transaction)
    }
    fn stake_owner_delegation_tx_encode_decode(transaction: Transaction<OwnerStakeDelegation>) -> TestResult {
        chain_test_utils::property::serialization_bijection_r(transaction)
    }
    /*
    fn certificate_tx_encode_decode(transaction: Transaction<Address, Certificate>) -> TestResult {
        chain_core::property::testing::serialization_bijection_r(transaction)
    }
    */
    fn signed_transaction_encode_decode(transaction: Transaction<NoExtra>) -> TestResult {
        chain_test_utils::property::serialization_bijection_r(transaction)
    }
}

#[cfg(test)]
fn check_eq<X>(s1: &str, x1: X, s2: &str, x2: X, s: &str) -> Result<(), String>
where
    X: Eq + std::fmt::Display,
{
    if x1 == x2 {
        Ok(())
    } else {
        Err(format!(
            "{} and {} have different number of {} : {} != {}",
            s1, s2, x1, x2, s
        ))
    }
}

#[quickcheck]
pub fn check_transaction_accessor_consistent(tx: Transaction<NoExtra>) -> TestResult {
    let slice = tx.as_slice();
    let res = check_eq(
        "tx",
        tx.nb_inputs(),
        "tx-slice",
        slice.nb_inputs(),
        "inputs",
    )
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_inputs(),
            "tx-inputs-slice",
            slice.inputs().nb_inputs(),
            "inputs",
        )
    })
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_inputs() as usize,
            "tx-inputs-slice-iter",
            slice.inputs().iter().count(),
            "inputs",
        )
    })
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_outputs(),
            "tx-outputs-slice",
            slice.outputs().nb_outputs(),
            "outputs",
        )
    })
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_outputs() as usize,
            "tx-outputs-slice-iter",
            slice.outputs().iter().count(),
            "outputs",
        )
    })
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_witnesses(),
            "tx-witness-slice",
            slice.witnesses().nb_witnesses(),
            "witnesses",
        )
    })
    .and_then(|()| {
        check_eq(
            "tx",
            tx.nb_witnesses() as usize,
            "tx-witness-slice-iter",
            slice.witnesses().iter().count(),
            "witnesses",
        )
    });
    match res {
        Ok(()) => TestResult::passed(),
        Err(e) => TestResult::error(e),
    }
}

impl Arbitrary for UtxoPointer {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        UtxoPointer {
            transaction_id: Arbitrary::arbitrary(g),
            output_index: Arbitrary::arbitrary(g),
            value: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Input {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Input::from_utxo(Arbitrary::arbitrary(g))
    }
}

impl Arbitrary for NoExtra {
    fn arbitrary<G: Gen>(_: &mut G) -> Self {
        Self
    }
}

impl<Extra: Arbitrary + Payload> Arbitrary for Transaction<Extra>
where
    Extra::Auth: Arbitrary,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let payload: Extra = Arbitrary::arbitrary(g);
        let payload_auth: Extra::Auth = Arbitrary::arbitrary(g);

        let num_inputs = u8::arbitrary(g) as usize;
        let num_outputs = u8::arbitrary(g) as usize;

        let inputs: Vec<_> = std::iter::repeat_with(|| Arbitrary::arbitrary(g))
            .take(num_inputs % 16)
            .collect();
        let outputs: Vec<_> = std::iter::repeat_with(|| Arbitrary::arbitrary(g))
            .take(num_outputs % 16)
            .collect();
        let witnesses: Vec<_> = std::iter::repeat_with(|| Arbitrary::arbitrary(g))
            .take(num_inputs % 16)
            .collect();

        TxBuilder::new()
            .set_payload(&payload)
            .set_ios(&inputs, &outputs)
            .set_witnesses(&witnesses)
            .set_payload_auth(&payload_auth)
    }
}

impl Arbitrary for SingleAccountBindingSignature {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        SingleAccountBindingSignature(Arbitrary::arbitrary(g))
    }
}

impl Arbitrary for AccountBindingSignature {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        AccountBindingSignature::Single(Arbitrary::arbitrary(g))
    }
}

#[derive(Clone)]
pub struct TransactionSigningKey(pub EitherEd25519SecretKey);

impl std::fmt::Debug for TransactionSigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TransactionSigningKey(<secret-key>)")
    }
}

impl Arbitrary for TransactionSigningKey {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        TransactionSigningKey(EitherEd25519SecretKey::Extended(arbitrary_secret_key(g)))
    }
}

impl Arbitrary for Witness {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let opt = u8::arbitrary(g) % 3;
        match opt {
            0 => Witness::Utxo(SpendingSignature::arbitrary(g)),
            1 => Witness::Account(SpendingSignature::arbitrary(g)),
            2 => {
                let sk: SecretKey<Ed25519> = arbitrary_secret_key(g);
                Witness::OldUtxo(sk.to_public(), [0u8; 32], Signature::arbitrary(g))
            }
            _ => panic!("not implemented"),
        }
    }
}

impl Arbitrary for UnspecifiedAccountIdentifier {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut b = [0u8; 32];
        for v in b.iter_mut() {
            *v = Arbitrary::arbitrary(g)
        }
        b.into()
    }
}

impl Arbitrary for AccountIdentifier {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        if Arbitrary::arbitrary(g) {
            AccountIdentifier::Single(Arbitrary::arbitrary(g))
        } else {
            AccountIdentifier::Multi(Arbitrary::arbitrary(g))
        }
    }
}
