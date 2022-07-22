use crate::snapshot::{registration::MainnetRewardAddress, SnapshotInfo};
use jormungandr_lib::crypto::{account::Identifier, hash::Hash};
use chain_addr::{Discrimination, Kind};
use chain_impl_mockchain::transaction::UnspecifiedAccountIdentifier;
use jormungandr_lib::{crypto::account::Identifier, interfaces::Address};
use rust_decimal::Decimal;
use snapshot_lib::{registration::MainnetRewardAddress, SnapshotInfo};
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

pub const ADA_TO_LOVELACE_FACTOR: u64 = 1_000_000;
pub type Rewards = Decimal;

pub struct Threshold {
    total: usize,
    per_challenge: HashMap<i32, usize>,
    proposals_per_challenge: HashMap<i32, HashSet<Hash>>,
}

impl Threshold {
    pub fn new(
        total_threshold: usize,
        per_challenge: HashMap<i32, usize>,
        proposals: Vec<FullProposalInfo>,
    ) -> Result<Self, Error> {
        let proposals = proposals
            .into_iter()
            .map(|p| {
                <[u8; 32]>::try_from(p.proposal.chain_proposal_id)
                    .map_err(Error::InvalidHash)
                    .map(|hash| (p.proposal.challenge_id, Hash::from(hash)))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            total: total_threshold,
            per_challenge,
            proposals_per_challenge: proposals.into_iter().fold(
                HashMap::new(),
                |mut acc, (challenge_id, hash)| {
                    acc.entry(challenge_id).or_default().insert(hash);
                    acc
                },
            ),
        })
    }

    fn filter(&self, votes: &HashSet<Hash>) -> bool {
        if votes.len() < self.total {
            return false;
        }

        for (challenge, threshold) in &self.per_challenge {
            let votes_in_challengs = self
                .proposals_per_challenge
                .get(challenge)
                .map(|props| votes.intersection(props).count())
                .unwrap_or_default();
            if votes_in_challengs < *threshold {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Value overflowed its maximum value")]
    Overflow,
    #[error("Multiple snapshot entries per voter are not supported")]
    MultipleEntries,
    #[error("Unknown voter group {0}")]
    UnknownVoterGroup(String),
    #[error("Invalid blake2b256 hash")]
    InvalidHash(Vec<u8>),
}

fn calculate_reward(
    total_stake: u64,
    stake_per_voter: HashMap<Identifier, u64>,
    total_rewards: Rewards,
) -> HashMap<Identifier, Rewards> {
    stake_per_voter
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                (Rewards::from(v) / Rewards::from(total_stake) * total_rewards),
            )
        })
        .collect()
}

pub type VoteCount = HashMap<Identifier, HashSet<Hash>>;

fn filter_active_addresses(
    vote_count: VoteCount,
    snapshot_info: Vec<SnapshotInfo>,
    threshold: Threshold,
) -> Vec<SnapshotInfo> {
    snapshot_info
        .into_iter()
        .filter(|v| {
            if let Some(votes) = vote_count.get(&v.hir.voting_key) {
                threshold.filter(votes)
            } else {
                threshold.filter(&HashSet::new())
            }
        })
        .collect()
}

pub fn account_hex_to_address(
    account_hex: String,
    discrimination: Discrimination,
) -> Result<Address, hex::FromHexError> {
    let mut buffer = [0u8; 32];
    hex::decode_to_slice(account_hex, &mut buffer)?;
    let identifier: UnspecifiedAccountIdentifier = UnspecifiedAccountIdentifier::from(buffer);
    Ok(Address::from(chain_addr::Address(
        discrimination,
        Kind::Account(
            identifier
                .to_single_account()
                .expect("Only single accounts are supported")
                .into(),
        ),
    )))
}

fn rewards_to_mainnet_addresses(
    rewards: HashMap<Identifier, Rewards>,
    voters: Vec<SnapshotInfo>,
) -> BTreeMap<MainnetRewardAddress, Rewards> {
    let mut res = BTreeMap::new();
    let snapshot_info_by_key = voters
        .into_iter()
        .map(|v| (v.hir.voting_key.clone(), v))
        .collect::<HashMap<_, _>>();
    for (addr, reward) in rewards {
        let contributions = snapshot_info_by_key
            .get(&addr)
            .map(|v| v.contributions.clone())
            .unwrap_or_default();
        let total_value = contributions
            .iter()
            .map(|c| Rewards::from(c.value))
            .sum::<Rewards>();

        for c in contributions {
            *res.entry(c.reward_address.clone()).or_default() +=
                reward * Rewards::from(c.value) / total_value;
        }
    }

    res
}

pub fn calc_voter_rewards(
    vote_count: VoteCount,
    voters: Vec<SnapshotInfo>,
    vote_threshold: Threshold,
    total_rewards: Rewards,
) -> Result<BTreeMap<MainnetRewardAddress, Rewards>, Error> {
    let unique_voters = voters
        .iter()
        .map(|s| s.hir.voting_key.clone())
        .collect::<HashSet<_>>();
    if unique_voters.len() != voters.len() {
        return Err(Error::MultipleEntries);
    }
    let active_addresses = filter_active_addresses(vote_count, voters, vote_threshold);

    let mut total_active_stake = 0u64;
    let mut stake_per_voter = HashMap::new();
    // iterative as Iterator::sum() panic on overflows
    for voter in &active_addresses {
        total_active_stake = total_active_stake
            .checked_add(voter.hir.voting_power.into())
            .ok_or(Error::Overflow)?;
        stake_per_voter.insert(voter.hir.voting_key.clone(), voter.hir.voting_power.into());
    }
    let rewards = calculate_reward(total_active_stake, stake_per_voter, total_rewards);
    Ok(rewards_to_mainnet_addresses(rewards, active_addresses))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::assert_are_close;
    use fraction::Fraction;
    use jormungandr_lib::crypto::account::Identifier;
    use snapshot_lib::registration::{Delegations, VotingRegistration};
    use snapshot_lib::Snapshot;
    use test_strategy::proptest;

    #[proptest]
    fn test_all_active(snapshot: Snapshot) {
        let votes_count = snapshot
            .voting_keys()
            .into_iter()
            .map(|key| (key.clone(), HashSet::from([Hash::from([0u8; 32])])))
            .collect::<VoteCount>();
        let n_voters = votes_count.len();
        let voters = snapshot.to_full_snapshot_info();
        let rewards = calc_voter_rewards(
            votes_count,
            voters,
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        if n_voters > 0 {
            assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE)
        } else {
            assert_eq!(rewards.len(), 0);
        }
    }

    #[proptest]
    fn test_all_inactive(snapshot: Snapshot) {
        let votes_count = VoteCount::new();
        let voters = snapshot.to_full_snapshot_info();
        let rewards = calc_voter_rewards(
            votes_count,
            voters,
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        assert_eq!(rewards.len(), 0);
    }

    #[proptest]
    fn test_small(snapshot: Snapshot) {
        let voting_keys = snapshot.voting_keys().collect::<Vec<_>>();

        let votes_count = voting_keys
            .iter()
            .enumerate()
            .map(|(i, &key)| {
                (
                    key.to_owned(),
                    if i % 2 == 0 {
                        HashSet::from([Hash::from([0u8; 32])])
                    } else {
                        HashSet::new()
                    },
                )
            })
            .collect::<VoteCount>();
        let n_voters = votes_count
            .iter()
            .filter(|(_, votes)| !votes.is_empty())
            .count();
        let voters = snapshot.to_full_snapshot_info();
        let voters_active = voters
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(i, _utxo)| i % 2 == 0)
            .map(|(_, utxo)| utxo)
            .collect::<Vec<_>>();

        let mut rewards = calc_voter_rewards(
            votes_count.clone(),
            voters,
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        let rewards_no_inactive = calc_voter_rewards(
            votes_count,
            voters_active,
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        // Rewards should ignore inactive voters
        assert_eq!(rewards, rewards_no_inactive);
        if n_voters > 0 {
            assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE);
        } else {
            assert_eq!(rewards.len(), 0);
        }

        let (active, inactive): (Vec<_>, Vec<_>) = voting_keys
            .into_iter()
            .enumerate()
            .partition(|(i, _vk)| i % 2 == 0);

        let active_reward_addresses = active
            .into_iter()
            .flat_map(|(_, vk)| {
                snapshot
                    .contributions_for_voting_key(vk.clone())
                    .into_iter()
                    .map(|c| c.reward_address)
            })
            .collect::<HashSet<_>>();

        assert!(active_reward_addresses
            .iter()
            .all(|addr| rewards.remove(addr).unwrap() > Rewards::ZERO));

        // partial test: does not check that rewards for addresses that delegated to both
        // active and inactive voters only come from active ones
        for (_, vk) in inactive {
            for contrib in snapshot.contributions_for_voting_key(vk.clone()) {
                assert!(rewards.get(&contrib.reward_address).is_none());
            }
        }
    }

    #[test]
    fn test_mapping() {
        let mut raw_snapshot = Vec::new();
        let voting_pub_key = Identifier::from_hex(&hex::encode([0; 32])).unwrap();

        let mut total_stake = 0u64;
        for i in 1..10u64 {
            let stake_public_key = i.to_string();
            let reward_address = i.to_string();
            let delegations = Delegations::New(vec![(voting_pub_key.clone(), 1)]);
            raw_snapshot.push(VotingRegistration {
                stake_public_key,
                voting_power: i.into(),
                reward_address,
                delegations,
                voting_purpose: 0,
            });
            total_stake += i;
        }

        let snapshot = Snapshot::from_raw_snapshot(
            raw_snapshot.into(),
            0.into(),
            Fraction::from(1u64),
            &|_vk: &Identifier| String::new(),
        )
        .unwrap();

        let voters = snapshot.to_full_snapshot_info();

        let rewards = calc_voter_rewards(
            VoteCount::new(),
            voters,
            Threshold::new(0, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        assert_eq!(rewards.values().sum::<Rewards>(), Rewards::ONE);
        for (addr, reward) in rewards {
            assert_eq!(
                reward,
                addr.parse::<Rewards>().unwrap() / Rewards::from(total_stake)
            );
        }
    }

    #[test]
    fn test_rewards_cap() {
        let mut raw_snapshot = Vec::new();

        for i in 1..10u64 {
            let voting_pub_key = Identifier::from_hex(&hex::encode([i as u8; 32])).unwrap();
            let stake_public_key = i.to_string();
            let reward_address = i.to_string();
            let delegations = Delegations::New(vec![(voting_pub_key.clone(), 1)]);
            raw_snapshot.push(VotingRegistration {
                stake_public_key,
                voting_power: i.into(),
                reward_address,
                delegations,
                voting_purpose: 0,
            });
        }

        let snapshot = Snapshot::from_raw_snapshot(
            raw_snapshot.into(),
            0.into(),
            Fraction::new(1u64, 9u64),
            &|_vk: &Identifier| String::new(),
        )
        .unwrap();

        let voters = snapshot.to_full_snapshot_info();

        let rewards = calc_voter_rewards(
            VoteCount::new(),
            voters,
            Threshold::new(0, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE);
        for (_, reward) in rewards {
            assert_eq!(reward, Rewards::ONE / Rewards::from(9u8));
        }
    }

    #[proptest]
    fn test_per_category_threshold(snapshot: Snapshot) {
        use vit_servicing_station_tests::common::data::ArbitrarySnapshotGenerator;

        let voters = snapshot.to_full_snapshot_info();

        let snapshot = ArbitrarySnapshotGenerator::default().snapshot();
        let mut proposals = snapshot.proposals();
        // for some reasone they are base64 encoded and truncatin is just easier
        for proposal in &mut proposals {
            proposal.proposal.chain_proposal_id.truncate(32);
        }
        let proposals_by_challenge =
            proposals
                .iter()
                .fold(<HashMap<_, Vec<_>>>::new(), |mut acc, prop| {
                    acc.entry(prop.proposal.challenge_id)
                        .or_default()
                        .push(Hash::from(
                            <[u8; 32]>::try_from(prop.proposal.chain_proposal_id.clone()).unwrap(),
                        ));
                    acc
                });
        let per_challenge_threshold = proposals_by_challenge
            .iter()
            .map(|(challenge, p)| (*challenge, p.len()))
            .collect::<HashMap<_, _>>();

        let mut votes_count = voters
            .iter()
            .map(|v| {
                (
                    v.hir.voting_key.clone(),
                    proposals_by_challenge
                        .values()
                        .flat_map(|p| p.iter())
                        .cloned()
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let (_, inactive) = votes_count.split_at_mut(voters.len() / 2);
        for v in inactive {
            v.1.remove(&v.1.iter().next().unwrap().clone());
        }

        let only_active = votes_count
            .clone()
            .into_iter()
            .take(voters.len() / 2)
            .collect::<HashMap<_, _>>();
        let votes_count = votes_count.into_iter().collect::<HashMap<_, _>>();

        let rewards = calc_voter_rewards(
            votes_count,
            voters.clone(),
            Threshold::new(1, per_challenge_threshold.clone(), proposals.clone()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();

        let rewards_only_active = calc_voter_rewards(
            only_active,
            voters,
            Threshold::new(1, per_challenge_threshold, proposals).unwrap(),
            Rewards::ONE,
        )
        .unwrap();

        assert_eq!(rewards_only_active, rewards);
    }
}
