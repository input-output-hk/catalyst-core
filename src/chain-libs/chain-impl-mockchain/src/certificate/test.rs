use super::*;
use crate::block::BlockDate;
use crate::fragment::ConfigParams;
use crate::ledger::governance::TreasuryGovernanceAction;
use crate::rewards::TaxType;
use crate::testing::data::CommitteeMembersManager;
use crate::vote;
use crate::{accounting::account::DelegationType, tokens::identifier::TokenIdentifier};
#[cfg(test)]
use chain_core::mempack::{ReadBuf, Readable};
use chain_crypto::{testing, Ed25519};
use chain_time::DurationSeconds;
use chain_vote::{Crs, EncryptedTally};
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use std::num::NonZeroU8;

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

impl Arbitrary for UpdateProposal {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut changes = ConfigParams::new();
        for _ in 0..u8::arbitrary(g) % 10 {
            changes.push(Arbitrary::arbitrary(g));
        }
        let proposer_id = UpdateProposerId::arbitrary(g);
        Self::new(changes, proposer_id)
    }
}

impl Arbitrary for UpdateVote {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let proposal_id = UpdateProposalId::arbitrary(g);
        let voter_id = UpdateVoterId::arbitrary(g);
        Self::new(proposal_id, voter_id)
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

        let mut keys = Vec::new();
        // it should have been 256 but is limited for the sake of adequate test times
        let keys_n = g.next_u32() % 15 + 1;
        let mut seed = [0u8; 32];
        g.fill_bytes(&mut seed);
        let mut rng = rand_chacha::ChaCha20Rng::from_seed(seed);
        let h = chain_vote::Crs::from_hash(&seed);
        for _i in 0..keys_n {
            let mc = chain_vote::MemberCommunicationKey::new(&mut rng);
            let threshold = 1;
            let m1 = chain_vote::MemberState::new(&mut rng, threshold, &h, &[mc.to_public()], 0);
            keys.push(m1.public_key());
        }

        let voting_token = TokenIdentifier::arbitrary(g);

        Self::new(
            vote_start,
            vote_end,
            committee_end,
            proposals,
            payload_type,
            keys,
            voting_token,
        )
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

fn arbitrary_decrypted_private_tally<G: Gen>(g: &mut G) -> DecryptedPrivateTally {
    let proposals_n = u8::arbitrary(g);
    let mut inner = Vec::new();
    let mut rng = ChaChaRng::seed_from_u64(u64::arbitrary(g));
    let crs_seed = String::arbitrary(g).into_bytes();
    let committee_size = (g.next_u32() % 2 + 1) as usize; // very time consuming
    let committee_manager =
        CommitteeMembersManager::new(&mut rng, &crs_seed, committee_size, committee_size);

    for _ in 0..proposals_n {
        let n_options = NonZeroU8::arbitrary(g);

        let encrypted_tally = EncryptedTally::new(
            n_options.get() as usize,
            committee_manager.election_pk(),
            Crs::from_hash(&crs_seed),
        );

        let mut decrypte_shares = Vec::new();
        for i in 0..committee_size {
            decrypte_shares.push(
                encrypted_tally
                    .partial_decrypt(&mut rng, committee_manager.members()[i].secret_key()),
            );
        }

        inner.push(DecryptedPrivateTallyProposal {
            tally_result: (0..n_options.get())
                .map(|_| u64::arbitrary(g))
                .collect::<Box<[_]>>(),
            decrypt_shares: decrypte_shares.into_boxed_slice(),
        });
    }
    DecryptedPrivateTally::new(inner).unwrap()
}

impl Arbitrary for VoteTally {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let vote_plan_id = VotePlanId::arbitrary(g);

        let private = bool::arbitrary(g);

        if private {
            Self::new_private(vote_plan_id, arbitrary_decrypted_private_tally(g))
        } else {
            Self::new_public(vote_plan_id)
        }
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

impl Arbitrary for EncryptedVoteTally {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let id = Arbitrary::arbitrary(g);
        Self::new(id)
    }
}

impl Arbitrary for EncryptedVoteTallyProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self {
            id: Arbitrary::arbitrary(g),
            signature: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Certificate {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let option = u8::arbitrary(g) % 12;
        match option {
            0 => Certificate::StakeDelegation(Arbitrary::arbitrary(g)),
            1 => Certificate::OwnerStakeDelegation(Arbitrary::arbitrary(g)),
            2 => Certificate::PoolRegistration(Arbitrary::arbitrary(g)),
            3 => Certificate::PoolRetirement(Arbitrary::arbitrary(g)),
            4 => Certificate::PoolUpdate(Arbitrary::arbitrary(g)),
            5 => Certificate::VotePlan(Arbitrary::arbitrary(g)),
            6 => Certificate::VoteCast(Arbitrary::arbitrary(g)),
            7 => Certificate::VoteTally(Arbitrary::arbitrary(g)),
            8 => Certificate::EncryptedVoteTally(Arbitrary::arbitrary(g)),
            9 => Certificate::UpdateProposal(Arbitrary::arbitrary(g)),
            10 => Certificate::UpdateVote(Arbitrary::arbitrary(g)),
            11 => Certificate::MintToken(Arbitrary::arbitrary(g)),
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
