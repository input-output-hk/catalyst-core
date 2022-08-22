use super::{Rewards, Threshold, VoteCount};
use crate::utils::assert_are_close;
use jormungandr_lib::crypto::account::Identifier;
use rust_decimal::Decimal;
use snapshot_lib::{SnapshotInfo, VotingGroup};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Value overflowed its maximum value")]
    Overflow,
    #[error("Multiple snapshot entries per voter are not supported")]
    MultipleEntries,
}

fn filter_requirements(
    mut dreps: Vec<SnapshotInfo>,
    votes: VoteCount,
    top_dreps_to_reward: usize,
    votes_threshold: Threshold,
) -> Vec<SnapshotInfo> {
    // only the top `top_dreps_to_reward` representatives get rewards
    dreps.sort_by_key(|v| v.hir.voting_power);
    dreps.reverse();
    dreps.truncate(top_dreps_to_reward);

    dreps
        .into_iter()
        .filter_map(|d| votes.get(&d.hir.voting_key).map(|d_votes| (d, d_votes)))
        .filter(|(_d, d_votes)| votes_threshold.filter(d_votes))
        .map(|(d, _d_votes)| d)
        .collect()
}

pub fn calc_dreps_rewards(
    snapshot: Vec<SnapshotInfo>,
    votes: VoteCount,
    drep_voting_group: VotingGroup,
    top_dreps_to_reward: usize,
    dreps_votes_threshold: Threshold,
    total_rewards: Decimal,
) -> Result<BTreeMap<Identifier, Rewards>, Error> {
    let total_active_stake = snapshot
        .iter()
        .try_fold(0u64, |acc, x| acc.checked_add(x.hir.voting_power.into()))
        .ok_or(Error::Overflow)?;

    let dreps = snapshot
        .into_iter()
        .filter(|v| v.hir.voting_group == drep_voting_group)
        .collect::<Vec<_>>();

    let total_dreps_stake = dreps
        .iter()
        .map(|d| u64::from(d.hir.voting_power))
        .sum::<u64>();

    let unique_dreps = dreps
        .iter()
        .map(|s| s.hir.voting_key.clone())
        .collect::<HashSet<_>>();
    if unique_dreps.len() != dreps.len() {
        return Err(Error::MultipleEntries);
    }

    let filtered = filter_requirements(dreps, votes, top_dreps_to_reward, dreps_votes_threshold);

    let res = filtered
        .into_iter()
        .map(|d| {
            let reward = Decimal::from(u64::from(d.hir.voting_power))
                / Decimal::from(total_active_stake)
                * total_rewards;
            (d.hir.voting_key, reward)
        })
        .collect::<BTreeMap<_, _>>();

    let expected_rewards = if total_active_stake == 0 {
        Decimal::ZERO
    } else {
        total_rewards * Decimal::from(total_dreps_stake) / Decimal::from(total_active_stake)
    };
    assert_are_close(res.values().sum(), expected_rewards);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jormungandr_lib::crypto::hash::Hash;
    use proptest::prop_assert_eq;
    use snapshot_lib::*;
    use std::collections::HashMap;
    use test_strategy::proptest;

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
        let voters = snapshot.to_full_snapshot_info();
        let voters_active = voters
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(i, _utxo)| i % 2 == 0)
            .map(|(_, utxo)| utxo)
            .collect::<Vec<_>>();

        let rewards = calc_dreps_rewards(
            voters,
            votes_count.clone(),
            String::new(),
            voting_keys.len(),
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        let rewards_no_inactive = calc_dreps_rewards(
            voters_active,
            votes_count.clone(),
            String::new(),
            voting_keys.len(),
            Threshold::new(1, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        // Rewards should ignore inactive voters
        prop_assert_eq!(rewards, rewards_no_inactive);
    }

    #[proptest]
    fn test_threshold(snapshot: Snapshot) {
        let voters = snapshot.to_full_snapshot_info();

        let rewards = calc_dreps_rewards(
            voters.clone(),
            VoteCount::new(),
            String::new(),
            1,
            Threshold::new(0, HashMap::new(), Vec::new()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();
        prop_assert_eq!(rewards.len(), std::cmp::min(1, voters.len()))
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

        let rewards = calc_dreps_rewards(
            voters.clone(),
            votes_count,
            String::new(),
            voters.len(),
            Threshold::new(1, per_challenge_threshold.clone(), proposals.clone()).unwrap(),
            Rewards::ONE,
        )
        .unwrap();

        let rewards_only_active = calc_dreps_rewards(
            voters.clone(),
            only_active,
            String::new(),
            voters.len(),
            Threshold::new(1, per_challenge_threshold, proposals).unwrap(),
            Rewards::ONE,
        )
        .unwrap();

        prop_assert_eq!(rewards_only_active, rewards);
    }
}
