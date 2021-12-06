use fixed::types::U64F64;

use chain_addr::{Discrimination, Kind};
use chain_impl_mockchain::transaction::UnspecifiedAccountIdentifier;
use std::collections::{HashMap, HashSet};

use jormungandr_lib::interfaces::{Address, Block0Configuration, Initial};

pub const ADA_TO_LOVELACE_FACTOR: u64 = 1_000_000;
pub type Rewards = U64F64;

pub fn calculate_stake<'address>(
    committee_keys: &HashSet<Address>,
    block0: &'address Block0Configuration,
) -> (u64, HashMap<&'address Address, u64>) {
    let mut total_stake: u64 = 0;
    let mut stake_per_voter: HashMap<&'address Address, u64> = HashMap::new();

    for fund in &block0.initial {
        match fund {
            Initial::Fund(fund) => {
                for utxo in fund {
                    if !committee_keys.contains(&utxo.address) {
                        let value: u64 = utxo.value.into();
                        total_stake += value;
                        let entry = stake_per_voter.entry(&utxo.address).or_default();
                        *entry += value;
                    }
                }
            }
            Initial::Cert(_) => {}
            Initial::LegacyFund(_) => {}
        }
    }
    (total_stake, stake_per_voter)
}

pub fn calculate_reward_share<'address>(
    total_stake: u64,
    stake_per_voter: &HashMap<&'address Address, u64>,
    threshold_addresses: &AddressesVoteCount,
    threshold: u64,
) -> HashMap<&'address Address, Rewards> {
    stake_per_voter
        .iter()
        .map(|(k, v)| {
            // if it doesnt appear in the votes count, it means it did not vote
            let reward = if *threshold_addresses.get(k).unwrap_or(&0u64) >= threshold {
                Rewards::from_num(*v) / total_stake as u128
            } else {
                Rewards::ZERO
            };
            (*k, reward)
        })
        .collect()
}

/// get the proportional reward from a share and total rewards amount
pub fn reward_from_share(share: Rewards, total_reward: u64) -> Rewards {
    Rewards::from_num(total_reward) * share
}

pub type VoteCount = HashMap<String, u64>;
pub type AddressesVoteCount = HashMap<Address, u64>;

pub fn vote_count_with_addresses(
    vote_count: VoteCount,
    block0: &Block0Configuration,
) -> AddressesVoteCount {
    let discrimination = block0.blockchain_configuration.discrimination;
    vote_count
        .into_iter()
        .map(|(account_hex, count)| {
            (
                account_hex_to_address(account_hex, discrimination)
                    .expect("Valid hex encoded UnspecifiedAccountIdentifier"),
                count,
            )
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
