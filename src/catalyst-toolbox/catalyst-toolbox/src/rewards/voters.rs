use crate::snapshot::{registration::MainnetRewardAddress, SnapshotInfo};
use jormungandr_lib::crypto::account::Identifier;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

pub const ADA_TO_LOVELACE_FACTOR: u64 = 1_000_000;
pub type Rewards = Decimal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Value overflowed its maximum value")]
    Overflow,
    #[error("Multiple snapshot entries per voter are not supported")]
    MultipleEntries,
    #[error("Unknown voter group {0}")]
    UnknownVoterGroup(String),
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

pub type VoteCount = HashMap<String, u64>;

fn filter_active_addresses(
    vote_count: VoteCount,
    threshold: u64,
    snapshot_info: Vec<SnapshotInfo>,
) -> Vec<SnapshotInfo> {
    snapshot_info
        .into_iter()
        .filter(|v| {
            let addr = v.hir.voting_key.to_hex();
            vote_count.get(&addr).copied().unwrap_or_default() >= threshold
        })
        .collect()
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
    vote_threshold: u64,
    voters: Vec<SnapshotInfo>,
    total_rewards: Rewards,
) -> Result<BTreeMap<MainnetRewardAddress, Rewards>, Error> {
    let unique_voters = voters
        .iter()
        .map(|s| s.hir.voting_key.clone())
        .collect::<HashSet<_>>();
    if unique_voters.len() != voters.len() {
        return Err(Error::MultipleEntries);
    }
    let active_addresses = filter_active_addresses(vote_count, vote_threshold, voters);

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
    use crate::snapshot::registration::*;
    use crate::snapshot::*;
    use crate::utils::assert_are_close;
    use fraction::Fraction;
    use jormungandr_lib::crypto::account::Identifier;
    use test_strategy::proptest;

    #[proptest]
    fn test_all_active(snapshot: Snapshot) {
        let votes_count = snapshot
            .voting_keys()
            .into_iter()
            .map(|key| (key.to_hex(), 1))
            .collect::<VoteCount>();
        let n_voters = votes_count.len();
        let voters = snapshot.to_full_snapshot_info();
        let rewards = calc_voter_rewards(votes_count, 1, voters, Rewards::ONE).unwrap();
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
        let rewards = calc_voter_rewards(votes_count, 1, voters, Rewards::ONE).unwrap();
        assert_eq!(rewards.len(), 0);
    }

    #[proptest]
    fn test_small(snapshot: Snapshot) {
        let voting_keys = snapshot.voting_keys().collect::<Vec<_>>();

        let votes_count = voting_keys
            .iter()
            .enumerate()
            .map(|(i, key)| (key.to_hex(), (i % 2 == 0) as u64))
            .collect::<VoteCount>();
        let n_voters = votes_count.iter().filter(|(_, votes)| **votes > 0).count();
        let voters = snapshot.to_full_snapshot_info();
        let voters_active = voters
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(i, _utxo)| i % 2 == 0)
            .map(|(_, utxo)| utxo)
            .collect::<Vec<_>>();

        let mut rewards = calc_voter_rewards(votes_count.clone(), 1, voters, Rewards::ONE).unwrap();
        let rewards_no_inactive =
            calc_voter_rewards(votes_count, 1, voters_active, Rewards::ONE).unwrap();
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

        let rewards = calc_voter_rewards(VoteCount::new(), 0, voters, Rewards::ONE).unwrap();
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

        let rewards = calc_voter_rewards(VoteCount::new(), 0, voters, Rewards::ONE).unwrap();
        assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE);
        for (_, reward) in rewards {
            assert_eq!(reward, Rewards::ONE / Rewards::from(9));
        }
    }
}
