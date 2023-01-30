use crate::{
    account::{DelegationType, Identifier},
    certificate::{
        Certificate, OwnerStakeDelegation, PoolId, PoolRegistration, PoolRetirement, PoolUpdate,
        StakeDelegation, UpdateProposal, UpdateProposalId, UpdateProposerId, UpdateVote,
        UpdateVoterId, VotePlanId, VoteTally,
    },
    config::ConfigParam,
    fragment::ConfigParams,
    testing::data::AddressData,
    transaction::UnspecifiedAccountIdentifier,
};
use chain_time::units::DurationSeconds;

pub fn build_stake_delegation_cert(
    stake_pool: &PoolRegistration,
    delegate_from: &AddressData,
) -> Certificate {
    let account_id = UnspecifiedAccountIdentifier::from_single_account(Identifier::from(
        delegate_from.delegation_key(),
    ));
    Certificate::StakeDelegation(StakeDelegation {
        account_id,
        delegation: DelegationType::Full(stake_pool.to_id()),
    })
}

pub fn build_stake_pool_registration_cert(stake_pool: &PoolRegistration) -> Certificate {
    Certificate::PoolRegistration(stake_pool.clone())
}

pub fn build_stake_pool_update_cert(stake_pool: &PoolUpdate) -> Certificate {
    Certificate::PoolUpdate(stake_pool.clone())
}

pub fn build_owner_stake_full_delegation(stake_pool: PoolId) -> Certificate {
    Certificate::OwnerStakeDelegation(OwnerStakeDelegation {
        delegation: DelegationType::Full(stake_pool),
    })
}

pub fn build_no_stake_delegation() -> Certificate {
    Certificate::OwnerStakeDelegation(OwnerStakeDelegation {
        delegation: DelegationType::NonDelegated,
    })
}

pub fn build_owner_stake_delegation(delegation_type: DelegationType) -> Certificate {
    Certificate::OwnerStakeDelegation(OwnerStakeDelegation {
        delegation: delegation_type,
    })
}

pub fn build_stake_pool_retirement_cert(pool_id: PoolId, start_validity: u64) -> Certificate {
    let retirement = PoolRetirement {
        pool_id,
        retirement_time: DurationSeconds(start_validity).into(),
    };

    Certificate::PoolRetirement(retirement)
}

pub fn build_vote_tally_cert(vote_id: VotePlanId) -> Certificate {
    Certificate::VoteTally(VoteTally::new_public(vote_id))
}

pub fn build_update_proposal_cert(
    proposer_id: UpdateProposerId,
    config_params: Vec<ConfigParam>,
) -> Certificate {
    let update_proposal = UpdateProposal::new(ConfigParams(config_params), proposer_id);
    Certificate::UpdateProposal(update_proposal)
}

pub fn build_update_vote_cert(
    proposal_id: UpdateProposalId,
    voter_id: UpdateVoterId,
) -> Certificate {
    let update_vote = UpdateVote::new(proposal_id, voter_id);
    Certificate::UpdateVote(update_vote)
}
