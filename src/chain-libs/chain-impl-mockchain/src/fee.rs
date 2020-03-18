use crate::certificate::CertificateSlice;
use crate::transaction as tx;
use crate::value::Value;
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;
use std::io::Error;
use std::num::NonZeroU64;

/// Linear fee using the basic affine formula
/// `COEFFICIENT * bytes(COUNT(tx.inputs) + COUNT(tx.outputs)) + CONSTANT + CERTIFICATE*COUNT(certificates)`.
#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
pub struct LinearFee {
    pub constant: u64,
    pub coefficient: u64,
    pub certificate: u64,
    pub per_certificate_fees: PerCertificateFee,
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct PerCertificateFee {
    pub certificate_pool_registration: Option<NonZeroU64>,
    pub certificate_stake_delegation: Option<NonZeroU64>,
    pub certificate_owner_stake_delegation: Option<NonZeroU64>,
}

impl LinearFee {
    pub fn new(constant: u64, coefficient: u64, certificate: u64) -> Self {
        LinearFee {
            constant,
            coefficient,
            certificate,
            per_certificate_fees: PerCertificateFee::default(),
        }
    }

    pub fn per_certificate_fees(&mut self, per_certificate_fees: PerCertificateFee) {
        self.per_certificate_fees = per_certificate_fees;
    }
}


fn pack_linear_fee<W: std::io::Write>(linear_fee: &LinearFee, codec: &mut Codec<W>) -> Result<(), std::io::Error> {
    codec.put_u64(linear_fee.constant)?;
    codec.put_u64(linear_fee.coefficient)?;
    codec.put_u64(linear_fee.certificate)?;
    pack_per_certificate_fee(linear_fee.per_certificate_fees, codec)?;
    Ok(())
}

fn unpack_linear_fee<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<LinearFee, std::io::Error> {
    let constant = codec.get_u64()?;
    let coefficient = codec.get_u64()?;
    let certificate = codec.get_u64()?;
    let per_certificate_fees = unpack_per_certificate_fee(codec)?;
    Ok(LinearFee {
        constant,
        coefficient,
        certificate,
        per_certificate_fees,
    })
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

    fn fees_for_certificate<'a>(&self, cert: CertificateSlice<'a>) -> Option<Value> {
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


fn pack_per_certificate_fee<W: std::io::Write>(per_certificate_fee : &PerCertificateFee, codec: &mut Codec<W>) -> Result<(), std::io::Error> {
    codec.put_u64(
        per_certificate_fee.certificate_pool_registration
            .map(|v| v.get())
            .unwrap_or(0),
    )?;
    codec.put_u64(
        per_certificate_fee.certificate_stake_delegation
            .map(|v| v.get())
            .unwrap_or(0),
    )?;
    codec.put_u64(
        per_certificate_fee.certificate_owner_stake_delegation
            .map(|v| v.get())
            .unwrap_or(0),
    )?;
    Ok(())
}


fn unpack_per_certificate_fee<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<PerCertificateFee, std::io::Error> {
    let certificate_pool_registration = std::num::NonZeroU64::new(codec.get_u64()?);
    let certificate_stake_delegation = std::num::NonZeroU64::new(codec.get_u64()?);
    let certificate_owner_stake_delegation = std::num::NonZeroU64::new(codec.get_u64()?);

    Ok(PerCertificateFee {
        certificate_pool_registration,
        certificate_stake_delegation,
        certificate_owner_stake_delegation,
    })
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
            tx.payload().to_certificate_slice(),
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
        self.per_certificate_fees
            .fees_for_certificate(cert_slice)
            .unwrap_or(Value(self.certificate))
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod test {
    use super::*;
    use crate::certificate::{Certificate, CertificatePayload};
    use chain_core::property::testing::serialization_bijection;
    use quickcheck::{Arbitrary, Gen, TestResult};
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

    impl Arbitrary for LinearFee {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self {
                constant: Arbitrary::arbitrary(g),
                coefficient: Arbitrary::arbitrary(g),
                certificate: Arbitrary::arbitrary(g),
                per_certificate_fees: PerCertificateFee::new(None, None, None),
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
    ) -> TestResult {
        fee.per_certificate_fees(per_certificate_fees);
        let per_certificate_fees = fee.per_certificate_fees;
        if per_certificate_fees.certificate_pool_registration.is_none()
            || per_certificate_fees.certificate_stake_delegation.is_none()
            || per_certificate_fees
                .certificate_owner_stake_delegation
                .is_none()
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

        match fee_value == expected_value {
            true => TestResult::passed(),
            false => TestResult::error(format!("Wrong fee: {} vs {}", fee_value, expected_value)),
        }
    }

    fn calculate_expected_cert_fee_value(certificate: &Certificate, fee: &LinearFee) -> u64 {
        let cert_fees = fee.per_certificate_fees;
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
            _ => fee.certificate,
        }
    }

    quickcheck! {
        fn per_certificate_fee_serialize_deserialize_bijection(b: PerCertificateFee) -> TestResult {
            serialization_bijection(b)
        }

        fn linear_fee_serialize_deserialize_bijection(b: LinearFee) -> TestResult {
            serialization_bijection(b)
        }
    }
}
