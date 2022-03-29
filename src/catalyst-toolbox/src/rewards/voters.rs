use crate::snapshot::{MainnetRewardAddress, Snapshot};
use chain_addr::{Discrimination, Kind};
use chain_impl_mockchain::transaction::UnspecifiedAccountIdentifier;
use chain_impl_mockchain::vote::CommitteeId;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

use jormungandr_lib::interfaces::{Address, Block0Configuration, Initial};

pub const ADA_TO_LOVELACE_FACTOR: u64 = 1_000_000;
pub type Rewards = Decimal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Value overflowed its maximum value")]
    Overflow,
}

fn calculate_active_stake<'address>(
    committee_keys: &HashSet<Address>,
    block0: &'address Block0Configuration,
    active_addresses: &ActiveAddresses,
) -> Result<(u64, HashMap<&'address Address, u64>), Error> {
    let mut total_stake: u64 = 0;
    let mut stake_per_voter: HashMap<&'address Address, u64> = HashMap::new();

    for fund in &block0.initial {
        match fund {
            Initial::Fund(fund) => {
                for utxo in fund {
                    // Exclude committee addresses (managed by IOG) and
                    // non active voters from total active stake for the purpose of calculating
                    // voter rewards
                    if !committee_keys.contains(&utxo.address)
                        && active_addresses.contains(&utxo.address)
                    {
                        let value: u64 = utxo.value.into();
                        total_stake = total_stake.checked_add(value).ok_or(Error::Overflow)?;
                        let entry = stake_per_voter.entry(&utxo.address).or_default();
                        *entry += value;
                    }
                }
            }
            Initial::Cert(_) => {}
            Initial::LegacyFund(_) => {}
            Initial::Token(_) => todo!("use the tokens for the initial stake?"),
        }
    }
    Ok((total_stake, stake_per_voter))
}

fn calculate_reward<'address>(
    total_stake: u64,
    stake_per_voter: &HashMap<&'address Address, u64>,
    active_addresses: &ActiveAddresses,
    total_rewards: Rewards,
) -> HashMap<&'address Address, Rewards> {
    stake_per_voter
        .iter()
        .map(|(k, v)| {
            let reward = if active_addresses.contains(k) {
                Rewards::from(*v) / Rewards::from(total_stake) * total_rewards
            } else {
                Rewards::ZERO
            };
            (*k, reward)
        })
        .collect()
}

pub type VoteCount = HashMap<String, u64>;
pub type ActiveAddresses = HashSet<Address>;

fn active_addresses(
    vote_count: VoteCount,
    block0: &Block0Configuration,
    threshold: u64,
    snapshot: &Snapshot,
) -> ActiveAddresses {
    let discrimination = block0.blockchain_configuration.discrimination;
    snapshot
        .voting_keys()
        // Add all keys from snapshot so that they are accounted for
        // even if they didn't vote and the threshold is 0.
        // Active accounts are overwritten with the correct count.
        .map(|key| (key.to_hex(), 0))
        .chain(vote_count.into_iter())
        .filter_map(|(account_hex, count)| {
            if count >= threshold {
                Some(
                    account_hex_to_address(account_hex, discrimination)
                        .expect("Valid hex encoded UnspecifiedAccountIdentifier"),
                )
            } else {
                None
            }
        })
        .collect()
}

fn account_hex_to_address(
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
    rewards: HashMap<&'_ Address, Rewards>,
    snapshot: Snapshot,
) -> BTreeMap<MainnetRewardAddress, Rewards> {
    let mut res = BTreeMap::new();
    for (addr, reward) in rewards {
        let registrations = snapshot.registrations_for_voting_key(
            addr.as_ref()
                .public_key()
                .expect("non account address")
                .clone(),
        );
        let total_stake = registrations
            .iter()
            .map(|reg| Rewards::from(u64::from(reg.voting_power)))
            .sum::<Rewards>();

        for reg in registrations {
            *res.entry(reg.reward_address.clone()).or_default() +=
                reward * Rewards::from(u64::from(reg.voting_power)) / total_stake;
        }
    }

    res
}

pub fn calc_voter_rewards(
    vote_count: VoteCount,
    vote_threshold: u64,
    block0: &Block0Configuration,
    snapshot: Snapshot,
    total_rewards: Rewards,
) -> Result<BTreeMap<MainnetRewardAddress, Rewards>, Error> {
    let active_addresses = active_addresses(vote_count, block0, vote_threshold, &snapshot);

    let committee_keys: HashSet<Address> = block0
        .blockchain_configuration
        .committees
        .iter()
        .cloned()
        .map(|id| {
            let id = CommitteeId::from(id);
            let pk = id.public_key();

            chain_addr::Address(Discrimination::Production, Kind::Account(pk)).into()
        })
        .collect();
    let (total_active_stake, stake_per_voter) =
        calculate_active_stake(&committee_keys, block0, &active_addresses)?;
    let rewards = calculate_reward(
        total_active_stake,
        &stake_per_voter,
        &active_addresses,
        total_rewards,
    );
    Ok(rewards_to_mainnet_addresses(rewards, snapshot))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::*;
    use crate::utils::assert_are_close;
    use chain_impl_mockchain::chaintypes::ConsensusVersion;
    use chain_impl_mockchain::fee::LinearFee;
    use jormungandr_lib::crypto::account::Identifier;
    use jormungandr_lib::interfaces::{BlockchainConfiguration, Stake};
    use jormungandr_lib::interfaces::{Initial, InitialUTxO};
    use test_strategy::proptest;

    fn blockchain_configuration(initial_funds: Vec<InitialUTxO>) -> Block0Configuration {
        Block0Configuration {
            blockchain_configuration: BlockchainConfiguration::new(
                Discrimination::Test,
                ConsensusVersion::Bft,
                LinearFee::new(1, 1, 1),
            ),
            initial: vec![Initial::Fund(initial_funds)],
        }
    }

    #[proptest]
    fn test_all_active(snapshot: Snapshot) {
        let votes_count = snapshot
            .voting_keys()
            .into_iter()
            .map(|key| (key.to_hex(), 1))
            .collect::<VoteCount>();
        let n_voters = votes_count.len();
        let initial = snapshot.to_block0_initials(Discrimination::Test);
        let block0 = blockchain_configuration(initial);
        let rewards = calc_voter_rewards(votes_count, 1, &block0, snapshot, Rewards::ONE).unwrap();
        if n_voters > 0 {
            assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE)
        } else {
            assert_eq!(rewards.len(), 0);
        }
    }

    #[proptest]
    fn test_all_inactive(snapshot: Snapshot) {
        let votes_count = VoteCount::new();
        let initial = snapshot.to_block0_initials(Discrimination::Test);
        let block0 = blockchain_configuration(initial);
        let rewards = calc_voter_rewards(votes_count, 1, &block0, snapshot, Rewards::ONE).unwrap();
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
        let initial = snapshot.to_block0_initials(Discrimination::Test);
        let block0 = blockchain_configuration(initial);
        let mut rewards =
            calc_voter_rewards(votes_count, 1, &block0, snapshot.clone(), Rewards::ONE).unwrap();
        if n_voters > 0 {
            assert_are_close(rewards.values().sum::<Rewards>(), Rewards::ONE);
        } else {
            assert_eq!(rewards.len(), 0);
        }

        for (i, vk) in voting_keys.into_iter().enumerate() {
            for reg in snapshot.registrations_for_voting_key(vk.clone()) {
                if i % 2 == 0 {
                    assert!(*rewards.get(&reg.reward_address).unwrap() > Rewards::ZERO);
                } else {
                    assert_eq!(
                        rewards.remove(&reg.reward_address).unwrap_or_default(),
                        Rewards::ZERO
                    );
                }
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
            let voting_power: Stake = i.into();
            raw_snapshot.push(CatalystRegistration {
                stake_public_key,
                voting_power,
                reward_address,
                voting_public_key: voting_pub_key.clone(),
            });
            total_stake += i;
        }

        let snapshot = Snapshot::from_raw_snapshot(raw_snapshot.into(), 0.into());

        let votes_count = snapshot
            .voting_keys()
            .into_iter()
            .map(|key| (key.to_hex(), 1))
            .collect::<VoteCount>();

        let initial = snapshot.to_block0_initials(Discrimination::Test);
        let block0 = blockchain_configuration(initial);

        let rewards = calc_voter_rewards(votes_count, 1, &block0, snapshot, Rewards::ONE).unwrap();
        assert_eq!(rewards.values().sum::<Rewards>(), Rewards::ONE);
        for (addr, reward) in rewards {
            assert_eq!(
                reward,
                addr.parse::<Rewards>().unwrap() / Rewards::from(total_stake)
            );
        }
    }
}
