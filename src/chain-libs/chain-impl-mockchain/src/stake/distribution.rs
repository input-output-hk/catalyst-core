use super::stake::Stake;
use crate::{
    account,
    accounting::account::DelegationType,
    certificate::{PoolId, PoolRegistration},
    utxo,
};
use chain_addr::{Address, Kind};
use std::collections::{hash_map, HashMap};
use std::sync::Arc;

use super::delegation::PoolsState;

/// Stake distribution at a given time.
///
/// Mainly containing the value associated with each pool,
/// but in future could also contains:
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeDistribution {
    // single address values
    pub unassigned: Stake,
    // group or account that doesn't have a valid stake pool assigned
    pub dangling: Stake,
    /// For each stake pool, the total stake value, and the value for the
    /// stake pool members.
    pub to_pools: HashMap<PoolId, PoolStakeInformation>,
}

impl StakeDistribution {
    pub fn get_total_stake(&self) -> Stake {
        Stake::sum(self.to_pools.values().map(|psi| psi.stake.total))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolStakeInformation {
    pub registration: Option<Arc<PoolRegistration>>,
    pub stake: PoolStakeDistribution,
}

impl PoolStakeInformation {
    pub fn add_value(&mut self, id: &account::Identifier, s: Stake) {
        self.stake.add(id.clone(), s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PoolStakeDistribution {
    pub total: Stake,
    pub accounts: HashMap<account::Identifier, Stake>,
}

impl PoolStakeDistribution {
    pub fn new() -> Self {
        Self {
            total: Stake::zero(),
            accounts: HashMap::new(),
        }
    }

    pub fn test_new_with_total_value(s: Stake) -> Self {
        Self {
            total: s,
            accounts: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: account::Identifier, s: Stake) {
        self.accounts.entry(id).and_modify(|c| *c += s).or_insert(s);
        self.total += s;
    }

    pub fn to_total(&self) -> Stake {
        Stake::sum(self.accounts.values().copied())
    }

    pub fn iter(&self) -> hash_map::Iter<'_, account::Identifier, Stake> {
        self.accounts.iter()
    }
}

impl StakeDistribution {
    pub fn empty() -> Self {
        StakeDistribution {
            unassigned: Stake::zero(),
            dangling: Stake::zero(),
            to_pools: HashMap::new(),
        }
    }

    /// Return the number of stake pools with non-zero stake.
    pub fn eligible_stake_pools(&self) -> usize {
        self.to_pools.len()
    }

    /// Return the total stake held by the eligible stake pools.
    pub fn total_stake(&self) -> Stake {
        Stake::sum(self.to_pools.values().map(|pool| pool.stake.total))
    }

    pub fn get_stake_for(&self, poolid: &PoolId) -> Option<Stake> {
        self.to_pools.get(poolid).map(|psd| psd.stake.total)
    }

    pub fn get_distribution(&self, pool_id: &PoolId) -> Option<&PoolStakeInformation> {
        self.to_pools.get(pool_id)
    }
}

fn assign_account_value(
    sd: &mut StakeDistribution,
    account_identifier: &account::Identifier,
    delegation_type: &DelegationType,
    value: Stake,
) {
    match delegation_type {
        DelegationType::NonDelegated => sd.unassigned += value,
        DelegationType::Full(ref pool_id) => {
            // if the pool exists, we add value to this pool distribution,
            // otherwise it get added to the dangling sum
            match sd.to_pools.get_mut(pool_id) {
                None => sd.dangling += value,
                Some(pool_info) => pool_info.add_value(account_identifier, value),
            }
        }
        DelegationType::Ratio(dr) => {
            // is the ratio distribution is not correct, considered it unassigned, otherwise
            // separate the total in as many parts as pools, and try to assign from the first to the last,
            // the stake associated plus if there's any remaining from the division.
            if dr.is_valid() {
                let sin = value.split_in(dr.parts() as u32);
                let mut r = sin.remaining;
                for (pool_id, ratio) in dr.pools().iter() {
                    let pool_value = sin.parts.scale(*ratio as u32);
                    match sd.to_pools.get_mut(pool_id) {
                        None => sd.dangling += pool_value,
                        Some(pool_info) => {
                            pool_info.add_value(account_identifier, pool_value + r);
                            r = Stake::zero();
                        }
                    }
                }
                // if r is not zero already, then we failed to assign it to anything, so just consider it as dangling
                if r > Stake::zero() {
                    sd.dangling += r
                }
            } else {
                sd.unassigned += value
            }
        }
    }
}

/// Calculate the Stake Distribution where the source of stake is coming from utxos and accounts,
/// and where the main targets is to calculate each value associated with *known* stake pools.
///
/// Everything that is linked to a stake pool that doesn't exist, will be added to dangling stake,
/// whereas all the utxo / accounts that doesn't have any delegation setup, will be counted towards
/// the unassigned stake.
pub fn get_distribution(
    accounts: &account::Ledger,
    dstate: &PoolsState,
    utxos: &utxo::Ledger<Address>,
) -> StakeDistribution {
    let mut distribution = StakeDistribution {
        unassigned: Stake::zero(),
        dangling: Stake::zero(),
        to_pools: dstate
            .stake_pools
            .iter()
            .map(|(id, pool_state)| {
                (
                    id.clone(),
                    PoolStakeInformation {
                        registration: Some(pool_state.registration.clone()),
                        stake: PoolStakeDistribution::new(),
                    },
                )
            })
            .collect(),
    };

    for (identifier, account_state) in accounts.iter() {
        assign_account_value(
            &mut distribution,
            identifier,
            account_state.delegation(),
            Stake::from_value(account_state.value()),
        )
    }

    for output in utxos.values() {
        // We're only interested in "group" addresses
        // (i.e. containing a spending key and a stake key).
        match output.address.kind() {
            Kind::Account(_) | Kind::Multisig(_) => {
                // single or multisig account are not present in utxos
                panic!("internal error: accounts in utxo")
            }
            Kind::Script(_) => {
                // scripts are not present in utxo
                panic!("internal error: script in utxo")
            }
            Kind::Group(_spending_key, account_key) => {
                let identifier = account_key.clone().into();
                // is there an account linked to this
                match accounts.get_state(&identifier) {
                    Err(err) => panic!(
                        "internal error: group's account should always be created: {:?}",
                        err
                    ),
                    Ok(st) => assign_account_value(
                        &mut distribution,
                        &identifier,
                        st.delegation(),
                        Stake::from_value(output.value),
                    ),
                }
            }
            Kind::Single(_) => distribution.unassigned += Stake::from_value(output.value),
        }
    }

    distribution
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account;
    use crate::accounting::account::DelegationType;
    use crate::stake::{delegation::PoolsState, Stake};
    use crate::{
        account::{AccountAlg, DelegationRatio, Identifier},
        certificate::PoolRegistration,
        fragment::FragmentId,
        testing::{
            arbitrary::{utils as arbitrary_utils, ArbitraryAddressDataValueVec},
            builders::StakePoolBuilder,
            data::{AddressData, AddressDataValue},
            TestGen,
        },
        transaction::{Output, TransactionIndex},
        utxo,
        value::Value,
    };
    use chain_addr::{Address, Kind};
    use chain_crypto::PublicKey;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    /// Holds all possible cases of distribution source
    #[derive(Clone, Debug)]
    pub struct StakeDistributionArbitraryData {
        utxos: Vec<(FragmentId, TransactionIndex, Output<Address>)>,
        unassigned_accounts: Vec<(Identifier, Value)>,
        assigned_accounts: Vec<(Identifier, Value)>,
        dangling_accounts: Vec<(Identifier, Value)>,
        groups: Vec<(FragmentId, TransactionIndex, Output<Address>)>,
        groups_single_account: Vec<(FragmentId, TransactionIndex, Output<Address>)>,
        single_account: (Identifier, Value),
        active_stake_pool: PoolRegistration,
        retired_stake_pool: PoolRegistration,
    }

    impl Arbitrary for StakeDistributionArbitraryData {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let arbitrary_addresses = ArbitraryAddressDataValueVec::arbitrary(gen);

            let utxos = arbitrary_addresses
                .utxos()
                .iter()
                .map(|x| {
                    (
                        FragmentId::arbitrary(gen),
                        TransactionIndex::arbitrary(gen),
                        x.make_output(),
                    )
                })
                .collect();

            let splitted_vec_of_accounts =
                arbitrary_utils::split_vec(&arbitrary_addresses.accounts(), gen, 3);

            let unassigned_accounts = splitted_vec_of_accounts
                .get(0)
                .unwrap()
                .iter()
                .map(|x| (Identifier::from(x.address_data.public_key()), x.value))
                .collect();

            let assigned_accounts = splitted_vec_of_accounts
                .get(1)
                .unwrap()
                .iter()
                .map(|x| (Identifier::from(x.address_data.public_key()), x.value))
                .collect();

            let dangling_accounts = splitted_vec_of_accounts
                .get(2)
                .unwrap()
                .iter()
                .map(|x| (Identifier::from(x.address_data.public_key()), x.value))
                .collect();

            let splitted_vec_of_delegations =
                arbitrary_utils::split_vec(&arbitrary_addresses.delegations(), gen, 2);

            let groups = splitted_vec_of_delegations
                .get(0)
                .unwrap()
                .iter()
                .map(|x| {
                    (
                        FragmentId::arbitrary(gen),
                        TransactionIndex::arbitrary(gen),
                        x.make_output(),
                    )
                })
                .collect();

            let single_account = (Identifier::arbitrary(gen), Value::arbitrary(gen));

            let groups_single_account = splitted_vec_of_delegations
                .get(1)
                .unwrap()
                .iter()
                .map(|x| {
                    AddressDataValue::new_with_tokens(
                        AddressData::delegation_for_account(
                            x.address_data.clone(),
                            single_account.0.clone().into(),
                        ),
                        x.value,
                        x.tokens.clone(),
                    )
                })
                .map(|x| {
                    (
                        FragmentId::arbitrary(gen),
                        TransactionIndex::arbitrary(gen),
                        x.make_output(),
                    )
                })
                .collect();

            let active_stake_pool = PoolRegistration::arbitrary(gen);
            let retired_stake_pool = PoolRegistration::arbitrary(gen);

            StakeDistributionArbitraryData {
                utxos,
                unassigned_accounts,
                assigned_accounts,
                dangling_accounts,
                groups,
                groups_single_account,
                single_account,
                active_stake_pool,
                retired_stake_pool,
            }
        }
    }

    impl StakeDistributionArbitraryData {
        pub fn calculate_unassigned(&self) -> Stake {
            self.get_sum_from_utxo_type(&self.utxos)
                + self.get_sum_from_account_type(&self.unassigned_accounts)
        }

        pub fn calculate_dangling(&self) -> Stake {
            self.get_sum_from_account_type(&self.dangling_accounts)
        }

        pub fn pools_total(&self) -> Stake {
            self.get_sum_from_account_type(&self.assigned_accounts)
                + self.get_sum_from_utxo_type(&self.groups)
                + self.get_sum_from_utxo_type(&self.groups_single_account)
                + Stake::from_value(self.single_account.1)
        }

        fn get_sum_from_utxo_type(
            &self,
            utxos: &[(FragmentId, TransactionIndex, Output<Address>)],
        ) -> Stake {
            Stake::sum(utxos.iter().map(|(_, _, x)| Stake::from_value(x.value)))
        }

        fn get_sum_from_account_type(&self, accounts: &[(Identifier, Value)]) -> Stake {
            Stake::sum(accounts.iter().map(|(_, x)| Stake::from_value(*x)))
        }
    }

    #[quickcheck]
    pub fn stake_distribution_is_consistent_with_total_value(
        stake_distribution_data: StakeDistributionArbitraryData,
    ) -> TestResult {
        let mut accounts = account::Ledger::new();
        let mut dstate = PoolsState::new();
        let mut utxos = utxo::Ledger::new();

        // create two stake pools, one active and one inactive
        let id_active_pool = stake_distribution_data.active_stake_pool.to_id();
        dstate = dstate
            .register_stake_pool(stake_distribution_data.active_stake_pool.clone())
            .unwrap();
        let id_retired_pool = stake_distribution_data.retired_stake_pool.to_id();

        // add utxos
        for (fragment_id, tx_index, output) in stake_distribution_data.utxos.iter().cloned() {
            utxos = utxos.add(&fragment_id, &[(tx_index, output)]).unwrap();
        }

        // add delegation addresses with all accounts delegated to active stake pool
        for (fragment_id, tx_index, output) in stake_distribution_data.groups.iter().cloned() {
            utxos = utxos
                .add(&fragment_id, &[(tx_index, output.clone())])
                .unwrap();
            let account_public_key: PublicKey<AccountAlg> = match output.address.kind() {
                Kind::Group(_, delegation_key) => delegation_key.clone(),
                _ => panic!("delegation utxo should have Kind::Group type"),
            };
            accounts = accounts
                .add_account(
                    Identifier::from(account_public_key.clone()),
                    Value::zero(),
                    (),
                )
                .unwrap();
            accounts = accounts
                .set_delegation(
                    &Identifier::from(account_public_key.clone()),
                    &DelegationType::Full(id_active_pool.clone()),
                )
                .unwrap();
        }

        // add delegation addresses which point to single account with delegation
        for (fragment_id, tx_index, output) in stake_distribution_data
            .groups_single_account
            .iter()
            .cloned()
        {
            utxos = utxos.add(&fragment_id, &[(tx_index, output)]).unwrap();
        }

        // add accounts without delegation
        for (id, value) in stake_distribution_data.unassigned_accounts.iter().cloned() {
            accounts = accounts.add_account(id, value, ()).unwrap();
        }

        // add accounts with delegation
        for (id, value) in stake_distribution_data.assigned_accounts.iter().cloned() {
            accounts = accounts.add_account(id.clone(), value, ()).unwrap();
            accounts = accounts
                .set_delegation(&id, &DelegationType::Full(id_active_pool.clone()))
                .unwrap();
        }

        // add accounts with delegation as a target for delegation addresses
        let single_account = stake_distribution_data.single_account.clone();
        accounts = accounts
            .add_account(single_account.0.clone(), single_account.1, ())
            .unwrap();
        accounts = accounts
            .set_delegation(&single_account.0, &DelegationType::Full(id_active_pool))
            .unwrap();

        // add accounts with retired stake pool
        for (id, value) in stake_distribution_data.dangling_accounts.iter().cloned() {
            accounts = accounts.add_account(id.clone(), value, ()).unwrap();
            accounts = accounts
                .set_delegation(&id, &DelegationType::Full(id_retired_pool.clone()))
                .unwrap();
        }

        // verify
        let distribution = super::get_distribution(&accounts, &dstate, &utxos);

        if distribution.unassigned != stake_distribution_data.calculate_unassigned() {
            return TestResult::error(format!(
                "Wrong Unassigned value. expected: {} but got {}",
                stake_distribution_data.calculate_unassigned(),
                &distribution.unassigned
            ));
        }

        if distribution.dangling != stake_distribution_data.calculate_dangling() {
            return TestResult::error(format!(
                "Wrong Unassigned value. expected: {} but got {}",
                stake_distribution_data.calculate_unassigned(),
                &distribution.unassigned
            ));
        }

        let pools_total_stake: Stake =
            Stake::sum(distribution.to_pools.values().map(|x| x.stake.total));
        if pools_total_stake != stake_distribution_data.pools_total() {
            return TestResult::error(format!(
                "Wrong Unassigned value. expected: {} but got {}",
                stake_distribution_data.pools_total(),
                pools_total_stake
            ));
        }
        TestResult::passed()
    }

    #[derive(Clone, Debug)]
    pub struct CorrectDelegationType(DelegationType);

    impl Arbitrary for CorrectDelegationType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let delegation_type = match u8::arbitrary(g) % 3u8 {
                0 => DelegationType::NonDelegated,
                1 => {
                    let pool_id = PoolRegistration::arbitrary(g).to_id();
                    DelegationType::Full(pool_id)
                }
                2 => {
                    let no_of_parts = 8u8;
                    let parts = arbitrary_utils::factorize(no_of_parts as u32, g);
                    let pool_parts: Vec<(PoolId, u8)> = parts
                        .iter()
                        .cloned()
                        .map(|x| (PoolRegistration::arbitrary(g).to_id(), x as u8))
                        .collect();

                    let delegation_ratio = DelegationRatio::new(no_of_parts, pool_parts)
                        .expect("incorrect delgation ratio");
                    DelegationType::Ratio(delegation_ratio)
                }
                _ => unreachable!(),
            };
            CorrectDelegationType(delegation_type)
        }
    }

    impl CorrectDelegationType {
        pub fn get_pools(&self) -> HashMap<PoolId, PoolStakeInformation> {
            match &self.0 {
                DelegationType::NonDelegated => HashMap::new(),
                DelegationType::Full(pool_id) => {
                    let mut pools = HashMap::new();
                    let information = PoolStakeInformation {
                        registration: None,
                        stake: PoolStakeDistribution::new(),
                    };
                    pools.insert(pool_id.clone(), information);
                    pools
                }
                DelegationType::Ratio(delegation_ratio) => {
                    let mut pools = HashMap::new();
                    for pool_id in delegation_ratio.pools().iter().cloned().map(|x| x.0) {
                        let information = PoolStakeInformation {
                            registration: None,
                            stake: PoolStakeDistribution::new(),
                        };
                        pools.insert(pool_id.clone(), information);
                    }
                    pools
                }
            }
        }
    }

    #[quickcheck]
    pub fn assign_account_value_is_consitent_with_stake_distribution(
        account_identifier: account::Identifier,
        delegation_type: CorrectDelegationType,
        value: Stake,
    ) -> TestResult {
        let mut stake_distribution = StakeDistribution::empty();
        stake_distribution.to_pools = delegation_type.get_pools();
        assign_account_value(
            &mut stake_distribution,
            &account_identifier,
            &delegation_type.0,
            value,
        );
        match delegation_type.0 {
            DelegationType::NonDelegated => {
                assert_distribution(stake_distribution, value, Stake::zero(), Stake::zero())
            }
            DelegationType::Full(_pool_id) => {
                assert_distribution(stake_distribution, Stake::zero(), Stake::zero(), value)
            }
            DelegationType::Ratio(_ratio) => {
                assert_distribution(stake_distribution, Stake::zero(), Stake::zero(), value)
            }
        }
    }

    pub fn assert_distribution(
        stake_distribution: StakeDistribution,
        unassigned: Stake,
        dangling: Stake,
        pools: Stake,
    ) -> TestResult {
        if stake_distribution.unassigned != unassigned {
            return TestResult::error(format!(
                "wrong unassigned {} vs {}",
                stake_distribution.unassigned, unassigned
            ));
        }
        if stake_distribution.dangling != dangling {
            return TestResult::error(format!(
                "wrong dangling {} vs {}",
                stake_distribution.dangling, dangling
            ));
        }
        if stake_distribution.total_stake() != pools {
            return TestResult::error(format!(
                "wrong to_pools {} vs {}",
                stake_distribution.total_stake(),
                pools
            ));
        }
        TestResult::passed()
    }

    #[test]
    pub fn dangling_stake_multiplied() {
        let mut stake_distribution = StakeDistribution::empty();
        let value = Value(10);
        let stake = Stake::from_value(value);
        let account_identifier = TestGen::identifier();
        let no_of_parts = 8u8;
        let parts = [1, 2, 2, 3];
        let pool_parts: Vec<(PoolId, u8)> = parts
            .iter()
            .cloned()
            .map(|x| (StakePoolBuilder::new().build().id(), x as u8))
            .collect();

        let delegation_ratio =
            DelegationRatio::new(no_of_parts, pool_parts).expect("incorrect delgation ratio");
        let delegation_type = DelegationType::Ratio(delegation_ratio);
        assign_account_value(
            &mut stake_distribution,
            &account_identifier,
            &delegation_type,
            stake,
        );
        assert_eq!(stake_distribution.dangling, stake);
    }
}
