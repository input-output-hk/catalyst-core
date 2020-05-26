use crate::certificate::CertificateSlice;
use crate::transaction as tx;
use crate::value::Value;
use std::num::NonZeroU64;

/// Linear fee using the basic affine formula
/// `COEFFICIENT * bytes(COUNT(tx.inputs) + COUNT(tx.outputs)) + CONSTANT + CERTIFICATE*COUNT(certificates)`.
#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
pub struct LinearFee {
    pub constant: u64,
    pub coefficient: u64,
    pub certificate: u64,
    pub per_certificate_fees: PerCertificateFee,
    pub per_vote_certificate_fees: PerVoteCertificateFee,
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct PerCertificateFee {
    pub certificate_pool_registration: Option<NonZeroU64>,
    pub certificate_stake_delegation: Option<NonZeroU64>,
    pub certificate_owner_stake_delegation: Option<NonZeroU64>,
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct PerVoteCertificateFee {
    pub certificate_vote_plan: Option<NonZeroU64>,
    pub certificate_vote_cast: Option<NonZeroU64>,
}

impl LinearFee {
    pub fn new(constant: u64, coefficient: u64, certificate: u64) -> Self {
        LinearFee {
            constant,
            coefficient,
            certificate,
            per_certificate_fees: PerCertificateFee::default(),
            per_vote_certificate_fees: PerVoteCertificateFee::default(),
        }
    }

    pub fn per_certificate_fees(&mut self, per_certificate_fees: PerCertificateFee) {
        self.per_certificate_fees = per_certificate_fees;
    }

    pub fn per_vote_certificate_fees(&mut self, per_vote_certificate_fees: PerVoteCertificateFee) {
        self.per_vote_certificate_fees = per_vote_certificate_fees;
    }
}

impl PerCertificateFee {
    pub fn new(
        certificate_pool_registration: Option<NonZeroU64>,
        certificate_stake_delegation: Option<NonZeroU64>,
        certificate_owner_stake_delegation: Option<NonZeroU64>,
    ) -> Self {
        Self {
            certificate_pool_registration,
            certificate_stake_delegation,
            certificate_owner_stake_delegation,
        }
    }

    fn fees_for_certificate<'a>(&self, cert: &CertificateSlice<'a>) -> Option<Value> {
        match cert {
            CertificateSlice::PoolRegistration(_) => {
                self.certificate_pool_registration.map(|v| Value(v.get()))
            }
            CertificateSlice::StakeDelegation(_) => {
                self.certificate_stake_delegation.map(|v| Value(v.get()))
            }
            CertificateSlice::OwnerStakeDelegation(_) => self
                .certificate_owner_stake_delegation
                .map(|v| Value(v.get())),
            _ => None,
        }
    }
}

impl PerVoteCertificateFee {
    pub fn new(
        certificate_vote_plan: Option<NonZeroU64>,
        certificate_vote_cast: Option<NonZeroU64>,
    ) -> Self {
        Self {
            certificate_vote_plan,
            certificate_vote_cast,
        }
    }

    fn fees_for_certificate<'a>(&self, cert: &CertificateSlice<'a>) -> Option<Value> {
        match cert {
            CertificateSlice::VotePlan(_) => self.certificate_vote_plan.map(|v| Value(v.get())),
            CertificateSlice::VoteCast(_) => self.certificate_vote_cast.map(|v| Value(v.get())),
            _ => None,
        }
    }
}

pub trait FeeAlgorithm {
    fn baseline(&self) -> Value;
    fn fees_for_inputs_outputs(&self, inputs: u8, outputs: u8) -> Value;
    fn fees_for_certificate<'a>(&self, cert: CertificateSlice<'a>) -> Value;

    fn calculate<'a>(&self, cert: Option<CertificateSlice<'a>>, inputs: u8, outputs: u8) -> Value {
        self.baseline()
            .saturating_add(self.fees_for_inputs_outputs(inputs, outputs))
            .saturating_add(cert.map_or(Value::zero(), |c| self.fees_for_certificate(c)))
    }

    fn calculate_tx<P: tx::Payload>(&self, tx: &tx::TransactionSlice<P>) -> Value {
        self.calculate(
            tx.payload().into_certificate_slice(),
            tx.nb_inputs(),
            tx.nb_outputs(),
        )
    }
}

impl FeeAlgorithm for LinearFee {
    fn baseline(&self) -> Value {
        Value(self.constant)
    }

    fn fees_for_inputs_outputs(&self, inputs: u8, outputs: u8) -> Value {
        Value(
            self.coefficient
                .saturating_mul((inputs as u64) + (outputs as u64)),
        )
    }

    fn fees_for_certificate<'a>(&self, cert_slice: CertificateSlice<'a>) -> Value {
        let f1 = self.per_certificate_fees.fees_for_certificate(&cert_slice);
        let f2 = self
            .per_vote_certificate_fees
            .fees_for_certificate(&cert_slice);
        f1.or(f2).unwrap_or(Value(self.certificate))
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod test {
    use super::*;
    #[cfg(test)]
    use crate::certificate::{Certificate, CertificatePayload};
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for PerCertificateFee {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            PerCertificateFee::new(
                NonZeroU64::new(u64::arbitrary(g)),
                NonZeroU64::new(u64::arbitrary(g)),
                NonZeroU64::new(u64::arbitrary(g)),
            )
        }
    }

    impl Arbitrary for PerVoteCertificateFee {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::new(
                NonZeroU64::new(u64::arbitrary(g)),
                NonZeroU64::new(u64::arbitrary(g)),
            )
        }
    }

    impl Arbitrary for LinearFee {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self {
                constant: Arbitrary::arbitrary(g),
                coefficient: Arbitrary::arbitrary(g),
                certificate: Arbitrary::arbitrary(g),
                per_certificate_fees: PerCertificateFee::new(None, None, None),
                per_vote_certificate_fees: PerVoteCertificateFee::new(None, None),
            }
        }
    }

    #[quickcheck]
    pub fn linear_fee_certificate_calculation(
        certificate: Certificate,
        inputs: u8,
        outputs: u8,
        mut fee: LinearFee,
        per_certificate_fees: PerCertificateFee,
        per_vote_certificate_fees: PerVoteCertificateFee,
    ) -> TestResult {
        fee.per_certificate_fees(per_certificate_fees);
        fee.per_vote_certificate_fees(per_vote_certificate_fees);
        let per_certificate_fees = fee.per_certificate_fees;
        if per_certificate_fees.certificate_pool_registration.is_none()
            || per_certificate_fees.certificate_stake_delegation.is_none()
            || per_certificate_fees
                .certificate_owner_stake_delegation
                .is_none()
            || per_vote_certificate_fees.certificate_vote_plan.is_none()
            || per_vote_certificate_fees.certificate_vote_cast.is_none()
        {
            return TestResult::discard();
        }

        let certificate_payload: CertificatePayload = (&certificate).into();
        let fee_value = fee.calculate(Some(certificate_payload.as_slice()), inputs, outputs);
        let inputs_outputs_fee: u64 = (inputs + outputs) as u64 * fee.coefficient;
        let expected_value = Value(
            calculate_expected_cert_fee_value(&certificate, &fee)
                + inputs_outputs_fee
                + fee.constant,
        );

        if fee_value == expected_value {
            TestResult::passed()
        } else {
            TestResult::error(format!("Wrong fee: {} vs {}", fee_value, expected_value))
        }
    }

    #[cfg(test)]
    fn calculate_expected_cert_fee_value(certificate: &Certificate, fee: &LinearFee) -> u64 {
        let cert_fees = fee.per_certificate_fees;
        let vote_cert_fees = fee.per_vote_certificate_fees;
        match certificate {
            Certificate::PoolRegistration { .. } => {
                cert_fees.certificate_pool_registration.unwrap().into()
            }
            Certificate::StakeDelegation { .. } => {
                cert_fees.certificate_stake_delegation.unwrap().into()
            }
            Certificate::OwnerStakeDelegation { .. } => {
                cert_fees.certificate_owner_stake_delegation.unwrap().into()
            }
            Certificate::VotePlan { .. } => vote_cert_fees.certificate_vote_plan.unwrap().into(),
            Certificate::VoteCast { .. } => vote_cert_fees.certificate_vote_cast.unwrap().into(),
            _ => fee.certificate,
        }
    }
}
