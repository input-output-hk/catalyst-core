mod delegation;
mod pool;
mod vote_cast;
mod vote_plan;
mod vote_tally;

#[cfg(any(test, feature = "property-test-api"))]
mod test;

use crate::transaction::{Payload, PayloadData, PayloadSlice};

pub use self::vote_cast::VoteCast;
pub use self::vote_plan::{
    ExternalProposalDocument, ExternalProposalId, Proposal, Proposals, PushProposal, VoteAction,
    VotePlan, VotePlanId, VotePlanProof,
};
pub use self::vote_tally::{TallyProof, VoteTally, VoteTallyPayload};
pub use delegation::{OwnerStakeDelegation, StakeDelegation};
pub use pool::{
    GenesisPraosLeaderHash, IndexSignatures, ManagementThreshold, PoolId, PoolOwnersSigned,
    PoolPermissions, PoolRegistration, PoolRegistrationHash, PoolRetirement, PoolSignature,
    PoolUpdate,
};

pub enum CertificateSlice<'a> {
    StakeDelegation(PayloadSlice<'a, StakeDelegation>),
    OwnerStakeDelegation(PayloadSlice<'a, OwnerStakeDelegation>),
    PoolRegistration(PayloadSlice<'a, PoolRegistration>),
    PoolRetirement(PayloadSlice<'a, PoolRetirement>),
    PoolUpdate(PayloadSlice<'a, PoolUpdate>),
    VotePlan(PayloadSlice<'a, VotePlan>),
    VoteCast(PayloadSlice<'a, VoteCast>),
    VoteTally(PayloadSlice<'a, VoteTally>),
}

impl<'a> From<PayloadSlice<'a, StakeDelegation>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, StakeDelegation>) -> CertificateSlice<'a> {
        CertificateSlice::StakeDelegation(payload)
    }
}

impl<'a> From<PayloadSlice<'a, OwnerStakeDelegation>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, OwnerStakeDelegation>) -> CertificateSlice<'a> {
        CertificateSlice::OwnerStakeDelegation(payload)
    }
}

impl<'a> From<PayloadSlice<'a, PoolRegistration>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, PoolRegistration>) -> CertificateSlice<'a> {
        CertificateSlice::PoolRegistration(payload)
    }
}
impl<'a> From<PayloadSlice<'a, PoolRetirement>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, PoolRetirement>) -> CertificateSlice<'a> {
        CertificateSlice::PoolRetirement(payload)
    }
}

impl<'a> From<PayloadSlice<'a, PoolUpdate>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, PoolUpdate>) -> CertificateSlice<'a> {
        CertificateSlice::PoolUpdate(payload)
    }
}

impl<'a> From<PayloadSlice<'a, VotePlan>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, VotePlan>) -> CertificateSlice<'a> {
        CertificateSlice::VotePlan(payload)
    }
}

impl<'a> From<PayloadSlice<'a, VoteCast>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, VoteCast>) -> CertificateSlice<'a> {
        CertificateSlice::VoteCast(payload)
    }
}

impl<'a> From<PayloadSlice<'a, VoteTally>> for CertificateSlice<'a> {
    fn from(payload: PayloadSlice<'a, VoteTally>) -> CertificateSlice<'a> {
        CertificateSlice::VoteTally(payload)
    }
}

impl<'a> CertificateSlice<'a> {
    pub fn into_owned(self) -> Certificate {
        match self {
            CertificateSlice::PoolRegistration(c) => {
                Certificate::PoolRegistration(c.into_payload())
            }
            CertificateSlice::PoolUpdate(c) => Certificate::PoolUpdate(c.into_payload()),
            CertificateSlice::PoolRetirement(c) => Certificate::PoolRetirement(c.into_payload()),
            CertificateSlice::StakeDelegation(c) => Certificate::StakeDelegation(c.into_payload()),
            CertificateSlice::OwnerStakeDelegation(c) => {
                Certificate::OwnerStakeDelegation(c.into_payload())
            }
            CertificateSlice::VotePlan(c) => Certificate::VotePlan(c.into_payload()),
            CertificateSlice::VoteCast(c) => Certificate::VoteCast(c.into_payload()),
            CertificateSlice::VoteTally(c) => Certificate::VoteTally(c.into_payload()),
        }
    }
}

#[derive(Clone)]
pub enum CertificatePayload {
    StakeDelegation(PayloadData<StakeDelegation>),
    OwnerStakeDelegation(PayloadData<OwnerStakeDelegation>),
    PoolRegistration(PayloadData<PoolRegistration>),
    PoolRetirement(PayloadData<PoolRetirement>),
    PoolUpdate(PayloadData<PoolUpdate>),
    VotePlan(PayloadData<VotePlan>),
    VoteCast(PayloadData<VoteCast>),
    VoteTally(PayloadData<VoteTally>),
}

impl CertificatePayload {
    pub fn as_slice(&self) -> CertificateSlice {
        match self {
            CertificatePayload::StakeDelegation(payload) => payload.borrow().into(),
            CertificatePayload::OwnerStakeDelegation(payload) => payload.borrow().into(),
            CertificatePayload::PoolRegistration(payload) => payload.borrow().into(),
            CertificatePayload::PoolRetirement(payload) => payload.borrow().into(),
            CertificatePayload::PoolUpdate(payload) => payload.borrow().into(),
            CertificatePayload::VotePlan(payload) => payload.borrow().into(),
            CertificatePayload::VoteCast(payload) => payload.borrow().into(),
            CertificatePayload::VoteTally(payload) => payload.borrow().into(),
        }
    }
}

impl<'a> From<&'a Certificate> for CertificatePayload {
    fn from(certificate: &'a Certificate) -> Self {
        match certificate {
            Certificate::StakeDelegation(payload) => {
                CertificatePayload::StakeDelegation(payload.payload_data())
            }
            Certificate::OwnerStakeDelegation(payload) => {
                CertificatePayload::OwnerStakeDelegation(payload.payload_data())
            }
            Certificate::PoolRegistration(payload) => {
                CertificatePayload::PoolRegistration(payload.payload_data())
            }
            Certificate::PoolRetirement(payload) => {
                CertificatePayload::PoolRetirement(payload.payload_data())
            }
            Certificate::PoolUpdate(payload) => {
                CertificatePayload::PoolUpdate(payload.payload_data())
            }
            Certificate::VotePlan(payload) => CertificatePayload::VotePlan(payload.payload_data()),
            Certificate::VoteCast(payload) => CertificatePayload::VoteCast(payload.payload_data()),
            Certificate::VoteTally(payload) => {
                CertificatePayload::VoteTally(payload.payload_data())
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Certificate {
    StakeDelegation(StakeDelegation),
    OwnerStakeDelegation(OwnerStakeDelegation),
    PoolRegistration(PoolRegistration),
    PoolRetirement(PoolRetirement),
    PoolUpdate(PoolUpdate),
    VotePlan(VotePlan),
    VoteCast(VoteCast),
    VoteTally(VoteTally),
}

impl From<StakeDelegation> for Certificate {
    fn from(cert: StakeDelegation) -> Certificate {
        Certificate::StakeDelegation(cert)
    }
}

impl From<OwnerStakeDelegation> for Certificate {
    fn from(cert: OwnerStakeDelegation) -> Certificate {
        Certificate::OwnerStakeDelegation(cert)
    }
}

impl From<PoolRegistration> for Certificate {
    fn from(cert: PoolRegistration) -> Certificate {
        Certificate::PoolRegistration(cert)
    }
}

impl From<PoolRetirement> for Certificate {
    fn from(cert: PoolRetirement) -> Certificate {
        Certificate::PoolRetirement(cert)
    }
}

impl From<PoolUpdate> for Certificate {
    fn from(cert: PoolUpdate) -> Certificate {
        Certificate::PoolUpdate(cert)
    }
}

impl From<VotePlan> for Certificate {
    fn from(vote_plan: VotePlan) -> Self {
        Self::VotePlan(vote_plan)
    }
}

impl From<VoteCast> for Certificate {
    fn from(vote_plan: VoteCast) -> Self {
        Self::VoteCast(vote_plan)
    }
}

impl From<VoteTally> for Certificate {
    fn from(vote_tally: VoteTally) -> Self {
        Self::VoteTally(vote_tally)
    }
}

impl Certificate {
    pub fn need_auth(&self) -> bool {
        match self {
            Certificate::PoolRegistration(_) => <PoolRegistration as Payload>::HAS_AUTH,
            Certificate::PoolUpdate(_) => <PoolUpdate as Payload>::HAS_AUTH,
            Certificate::PoolRetirement(_) => <PoolRetirement as Payload>::HAS_AUTH,
            Certificate::StakeDelegation(_) => <StakeDelegation as Payload>::HAS_AUTH,
            Certificate::OwnerStakeDelegation(_) => <OwnerStakeDelegation as Payload>::HAS_AUTH,
            Certificate::VotePlan(_) => <VotePlan as Payload>::HAS_AUTH,
            Certificate::VoteCast(_) => <VoteCast as Payload>::HAS_AUTH,
            Certificate::VoteTally(_) => <VoteTally as Payload>::HAS_AUTH,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SignedCertificate {
    StakeDelegation(StakeDelegation, <StakeDelegation as Payload>::Auth),
    OwnerStakeDelegation(
        OwnerStakeDelegation,
        <OwnerStakeDelegation as Payload>::Auth,
    ),
    PoolRegistration(PoolRegistration, <PoolRegistration as Payload>::Auth),
    PoolRetirement(PoolRetirement, <PoolRetirement as Payload>::Auth),
    PoolUpdate(PoolUpdate, <PoolUpdate as Payload>::Auth),
    VotePlan(VotePlan, <VotePlan as Payload>::Auth),
    VoteTally(VoteTally, <VoteTally as Payload>::Auth),
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    pub fn need_auth(certificate: Certificate) -> TestResult {
        let expected_result = match certificate {
            Certificate::PoolRegistration(_) => true,
            Certificate::PoolUpdate(_) => true,
            Certificate::PoolRetirement(_) => true,
            Certificate::StakeDelegation(_) => true,
            Certificate::OwnerStakeDelegation(_) => false,
            Certificate::VotePlan(_) => true,
            Certificate::VoteCast(_) => false,
            Certificate::VoteTally(_) => true,
        };
        TestResult::from_bool(certificate.need_auth() == expected_result)
    }
}
