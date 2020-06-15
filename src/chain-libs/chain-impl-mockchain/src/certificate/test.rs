use super::*;
use crate::accounting::account::DelegationType;
use crate::block::BlockDate;
use crate::ledger::governance::TreasuryGovernanceAction;
use crate::rewards::TaxType;
use crate::vote;
#[cfg(test)]
use chain_core::mempack::{ReadBuf, Readable};
use chain_crypto::{testing, Ed25519};
use chain_time::DurationSeconds;
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

impl Arbitrary for PoolRetirement {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let retirement_time = DurationSeconds::from(u64::arbitrary(g)).into();
        PoolRetirement {
            pool_id: Arbitrary::arbitrary(g),
            retirement_time,
        }
    }
}

impl Arbitrary for PoolUpdate {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let pool_id = Arbitrary::arbitrary(g);
        let last_pool_reg_hash = Arbitrary::arbitrary(g);
        let new_pool_reg = Arbitrary::arbitrary(g);

        PoolUpdate {
            pool_id,
            last_pool_reg_hash,
            new_pool_reg,
        }
    }
}

impl Arbitrary for PoolOwnersSigned {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut signatoree = u8::arbitrary(g) % 32;
        if signatoree == 0 {
            signatoree = 1;
        }

        let mut signatures = Vec::new();
        for i in 0..signatoree {
            let s = Arbitrary::arbitrary(g);
            signatures.push((i, s));
        }
        PoolOwnersSigned { signatures }
    }
}

impl Arbitrary for PoolSignature {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        if bool::arbitrary(g) {
            PoolSignature::Operator(Arbitrary::arbitrary(g))
        } else {
            PoolSignature::Owners(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for PoolPermissions {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        PoolPermissions::new(u8::arbitrary(g))
    }
}

impl Arbitrary for DelegationType {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        DelegationType::Full(Arbitrary::arbitrary(g))
    }
}

impl Arbitrary for StakeDelegation {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        StakeDelegation {
            account_id: Arbitrary::arbitrary(g),
            delegation: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for OwnerStakeDelegation {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self {
            delegation: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for PoolRegistration {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let start_validity: DurationSeconds = u64::arbitrary(g).into();
        let keys = Arbitrary::arbitrary(g);

        let nb_owners = usize::arbitrary(g) % 32;
        let nb_operators = usize::arbitrary(g) % 4;

        let mut owners = Vec::new();
        for _ in 0..nb_owners {
            let pk = testing::arbitrary_public_key::<Ed25519, G>(g);
            owners.push(pk)
        }

        let mut operators = Vec::new();
        for _ in 0..nb_operators {
            let pk = testing::arbitrary_public_key::<Ed25519, G>(g);
            operators.push(pk)
        }

        PoolRegistration {
            serial: Arbitrary::arbitrary(g),
            permissions: PoolPermissions::new(1),
            start_validity: start_validity.into(),
            owners,
            operators: operators.into(),
            rewards: TaxType::zero(),
            reward_account: None,
            keys,
        }
    }
}

impl Arbitrary for TreasuryGovernanceAction {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        TreasuryGovernanceAction::TransferToRewards {
            value: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for VoteAction {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        if let Some(action) = Arbitrary::arbitrary(g) {
            VoteAction::Treasury { action }
        } else {
            VoteAction::OffChain
        }
    }
}

impl Arbitrary for Proposal {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let external_id = ExternalProposalId::arbitrary(g);
        let funding_plan = vote::Options::arbitrary(g);
        let action = VoteAction::arbitrary(g);

        Self::new(external_id, funding_plan, action)
    }
}

impl Arbitrary for Proposals {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let len = usize::arbitrary(g) % Proposals::MAX_LEN;
        let mut proposals = Proposals::new();
        for _ in 0..len {
            if let PushProposal::Success = proposals.push(Proposal::arbitrary(g)) {
                // pushed successfully
            } else {
                unreachable!("only generates what is needed")
            }
        }

        proposals
    }
}

impl Arbitrary for VotePlan {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vote_start = BlockDate::arbitrary(g);
        let vote_end = BlockDate::arbitrary(g);
        let committee_end = BlockDate::arbitrary(g);
        let proposals = Proposals::arbitrary(g);
        let payload_type = vote::PayloadType::arbitrary(g);

        Self::new(vote_start, vote_end, committee_end, proposals, payload_type)
    }
}

impl Arbitrary for VotePlanProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self {
            id: Arbitrary::arbitrary(g),
            signature: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for VoteCast {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vote_plan = VotePlanId::arbitrary(g);
        let proposal_index = u8::arbitrary(g);
        let payload = vote::Payload::arbitrary(g);

        VoteCast::new(vote_plan, proposal_index, payload)
    }
}

impl Arbitrary for VoteTally {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vote_plan_id = VotePlanId::arbitrary(g);
        Self::new_public(vote_plan_id)
    }
}

impl Arbitrary for TallyProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self::Public {
            id: Arbitrary::arbitrary(g),
            signature: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Certificate {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let option = u8::arbitrary(g) % 8;
        match option {
            0 => Certificate::StakeDelegation(Arbitrary::arbitrary(g)),
            1 => Certificate::OwnerStakeDelegation(Arbitrary::arbitrary(g)),
            2 => Certificate::PoolRegistration(Arbitrary::arbitrary(g)),
            3 => Certificate::PoolRetirement(Arbitrary::arbitrary(g)),
            4 => Certificate::PoolUpdate(Arbitrary::arbitrary(g)),
            5 => Certificate::VotePlan(Arbitrary::arbitrary(g)),
            6 => Certificate::VoteCast(Arbitrary::arbitrary(g)),
            7 => Certificate::VoteTally(Arbitrary::arbitrary(g)),
            _ => panic!("unimplemented"),
        }
    }
}

#[quickcheck]
fn pool_reg_serialization_bijection(b: PoolRegistration) -> TestResult {
    let b_got = b.serialize();
    let mut buf = ReadBuf::from(b_got.as_ref());
    let result = PoolRegistration::read(&mut buf);
    let left = Ok(b);
    assert_eq!(left, result);
    assert_eq!(buf.get_slice_end(), &[]);
    TestResult::from_bool(left == result)
}
