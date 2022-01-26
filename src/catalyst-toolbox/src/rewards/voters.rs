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
            Initial::Token(_) => todo!("use the tokens for the initial stake?"),
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

#[cfg(test)]
mod test {
    use super::*;
    use jormungandr_automation::jormungandr::ConfigurationBuilder;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use thor::Wallet;

    #[quickcheck]
    pub fn test_calculate_stake(wallet_count: usize, committe_count: usize) -> TestResult {
        if committe_count >= wallet_count {
            return TestResult::discard();
        }

        let wallets: Vec<Wallet> = std::iter::from_fn(|| Some(Default::default()))
            .take(wallet_count)
            .collect();
        let wallet_initial_funds = 10;

        let block0 = ConfigurationBuilder::new()
            .with_funds(
                wallets
                    .iter()
                    .map(|x| x.to_initial_fund(wallet_initial_funds))
                    .collect(),
            )
            .build_block0();

        let committees = wallets
            .iter()
            .take(committe_count)
            .map(|x| x.address())
            .collect();

        let (total_stake, addresses) = calculate_stake(&committees, &block0);

        for (_, value) in addresses {
            assert_eq!(value, wallet_initial_funds);
        }
        TestResult::from_bool(
            total_stake == ((wallet_count - committe_count) as u64 * wallet_initial_funds),
        )
    }

    #[test]
    pub fn test_calculate_rewards() {
        let wallet_count = 10usize;

        let wallets: Vec<Wallet> = std::iter::from_fn(|| Some(Default::default()))
            .take(wallet_count)
            .collect();
        let wallet_initial_funds = 1;

        let addresses: AddressesVoteCount = wallets
            .iter()
            .take(wallet_count)
            .map(|x| (x.address(), wallet_initial_funds))
            .collect();

        let rewards = calculate_reward_share(
            wallet_initial_funds * wallet_count as u64,
            &addresses.iter().map(|(x, y)| (x, *y)).collect(),
            &wallets
                .iter()
                .take(wallet_count)
                .map(|x| (x.address(), 1))
                .collect(),
            1,
        );

        for (_, value) in rewards {
            assert_eq!((value * 10).round(), Rewards::from_num(1));
        }
    }
}
