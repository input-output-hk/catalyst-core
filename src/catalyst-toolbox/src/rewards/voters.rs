use fixed::types::U64F64;
use jormungandr_lib::interfaces::{Address, Block0Configuration, Initial};
use std::collections::{HashMap, HashSet};

pub const ADA_TO_LOVELACE_FACTOR: u64 = 1_000_000;

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
) -> HashMap<&'address Address, U64F64> {
    stake_per_voter
        .iter()
        .map(|(k, v)| (*k, fixed::types::U64F64::from_num(*v) / total_stake as u128))
        .collect()
}

/// get the proportional reward from a share and total rewards amount
pub fn reward_from_share(share: U64F64, total_reward: u64) -> fixed::types::U64F64 {
    fixed::types::U64F64::from_num(total_reward) * share
}
