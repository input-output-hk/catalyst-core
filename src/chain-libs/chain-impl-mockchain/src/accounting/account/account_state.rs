use crate::date::Epoch;
use crate::value::*;
use crate::{certificate::PoolId, tokens::identifier::TokenIdentifier};
use imhamt::{Hamt, HamtIter};
use std::collections::hash_map::DefaultHasher;

use super::spending::{SpendingCounter, SpendingCounterIncreasing};
use super::{LastRewards, LedgerError};

/// Set the choice of delegation:
///
/// * No delegation
/// * Full delegation of this account to a specific pool
/// * Ratio of stake to multiple pools
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DelegationType {
    NonDelegated,
    Full(PoolId),
    Ratio(DelegationRatio),
}

/// Delegation Ratio type express a number of parts
/// and a list of pools and their individual parts
///
/// E.g. parts: 7, pools: [(A,2), (B,1), (C,4)] means that
/// A is associated with 2/7 of the stake, B has 1/7 of stake and C
/// has 4/7 of the stake.
///
/// It's invalid to have less than 2 elements in the array,
/// and by extension parts need to be equal to the sum of individual
/// pools parts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DelegationRatio {
    pub(crate) parts: u8,
    pub(crate) pools: Box<[(PoolId, u8)]>,
}

/// The maximum number of pools
pub const DELEGATION_RATIO_MAX_DECLS: usize = 8;

impl DelegationRatio {
    pub fn is_valid(&self) -> bool {
        // map to u32 before summing to make sure we don't overflow
        let total: u32 = self.pools.iter().map(|x| x.1 as u32).sum();
        let has_no_zero = !self.pools.iter().any(|x| x.1 == 0);
        has_no_zero
            && self.parts > 1
            && self.pools.len() > 1
            && self.pools.len() <= DELEGATION_RATIO_MAX_DECLS
            && total == (self.parts as u32)
    }

    pub fn new(parts: u8, pools: Vec<(PoolId, u8)>) -> Option<DelegationRatio> {
        let total: u32 = pools.iter().map(|x| x.1 as u32).sum();
        let has_no_zero = !pools.iter().any(|x| x.1 == 0);
        if has_no_zero
            && parts > 1
            && pools.len() > 1
            && pools.len() <= DELEGATION_RATIO_MAX_DECLS
            && total == (parts as u32)
        {
            Some(Self {
                parts,
                pools: pools.into(),
            })
        } else {
            None
        }
    }

    pub fn parts(&self) -> u8 {
        self.parts
    }

    pub fn pools(&self) -> &[(PoolId, u8)] {
        &self.pools
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AccountState<Extra> {
    pub spending: SpendingCounterIncreasing,
    pub delegation: DelegationType,
    pub value: Value,
    pub tokens: Hamt<DefaultHasher, TokenIdentifier, Value>,
    pub last_rewards: LastRewards,
    pub extra: Extra,
}

impl<Extra> AccountState<Extra> {
    /// Create a new account state with a specific start value
    pub fn new(v: Value, e: Extra) -> Self {
        Self {
            spending: SpendingCounterIncreasing::default(),
            delegation: DelegationType::NonDelegated,
            value: v,
            tokens: Hamt::new(),
            last_rewards: LastRewards::default(),
            extra: e,
        }
    }

    pub fn new_reward(epoch: Epoch, v: Value, extra: Extra) -> Self {
        let mut st = Self::new(v, extra);
        st.last_rewards.add_for(epoch, v);
        st
    }

    /// Get referencet to delegation setting
    pub fn delegation(&self) -> &DelegationType {
        &self.delegation
    }

    pub fn value(&self) -> Value {
        self.value
    }
}

impl<Extra: Clone> AccountState<Extra> {
    /// Same as add() except use a ValueError
    pub fn add_value(&self, v: Value) -> Result<Self, ValueError> {
        let new_value = (self.value + v)?;
        let mut st = self.clone();
        st.value = new_value;
        Ok(st)
    }

    /// Add a value to an account state
    ///
    /// Only error if value is overflowing
    pub fn add(&self, v: Value) -> Result<Self, LedgerError> {
        let new_value = (self.value + v)?;
        let mut st = self.clone();
        st.value = new_value;
        Ok(st)
    }

    /// Add Rewards to the account value but also as the last_reward
    pub fn add_rewards(&self, e: Epoch, v: Value) -> Result<Self, ValueError> {
        let new_value = (self.value + v)?;
        let mut st = self.clone();
        st.value = new_value;
        st.last_rewards.add_for(e, v);
        Ok(st)
    }

    /// Subtract a value from an account state, and return the new state.
    ///
    /// Note that this *also* increment the counter, as this function would be usually call
    /// for spending.
    pub fn sub(&self, spending: SpendingCounter, v: Value) -> Result<Option<Self>, LedgerError> {
        let new_value = (self.value - v)?;
        let mut r = self.clone();
        r.spending.next_verify(spending)?;
        r.value = new_value;
        Ok(Some(r))
    }

    /// Add a value to a token in an account state
    ///
    /// Only error if value is overflowing
    pub fn token_add(&self, token: TokenIdentifier, v: Value) -> Result<Self, LedgerError> {
        let tokens = self
            .tokens
            .insert_or_update(token, v, |current_value| (*current_value + v).map(Some))?;
        Ok(Self {
            tokens,
            ..self.clone()
        })
    }

    /// Set delegation
    pub fn set_delegation(&self, delegation: DelegationType) -> Self {
        let mut st = self.clone();
        st.delegation = delegation;
        st
    }
}

pub struct Iter<'a, ID, Extra>(pub HamtIter<'a, ID, AccountState<Extra>>);

impl<'a, ID, Extra> Iterator for Iter<'a, ID, Extra> {
    type Item = (&'a ID, &'a AccountState<Extra>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccountState, DelegationRatio, DelegationType, LastRewards, SpendingCounter,
        SpendingCounterIncreasing, DELEGATION_RATIO_MAX_DECLS,
    };
    use crate::{certificate::PoolId, testing::builders::StakePoolBuilder, value::Value};
    use imhamt::Hamt;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;
    use std::iter;

    #[quickcheck]
    pub fn account_sub_is_consistent(
        init_value: Value,
        sub_value: Value,
        counter: u32,
    ) -> TestResult {
        let mut account_state = AccountState::new(init_value, ());
        let counter = SpendingCounter::from(counter);
        account_state.spending = SpendingCounterIncreasing::new_from_counter(counter);
        TestResult::from_bool(
            should_sub_fail(account_state.clone(), sub_value)
                == account_state.sub(counter, sub_value).is_err(),
        )
    }

    #[quickcheck]
    pub fn add_value(init_value: Value, value_to_add: Value) -> TestResult {
        let account_state = AccountState::new(init_value, ());
        let left = account_state.add_value(value_to_add);
        let right = account_state.add(value_to_add);
        match (left, right) {
            (Err(_), Err(_)) => TestResult::passed(),
            (Ok(next_left), Ok(next_right)) => {
                TestResult::from_bool(next_left.value() == next_right.value())
            }
            (Ok(_), Err(_)) => TestResult::error("add_value() success while add() failed"),
            (Err(_), Ok(_)) => TestResult::error("add() success while add_value() failed"),
        }
    }

    #[derive(Clone, Debug)]
    pub enum ArbitraryAccountStateOp {
        Add(Value),
        Sub(Value),
        Delegate(PoolId),
        RemoveDelegation,
    }

    impl Arbitrary for ArbitraryAccountStateOp {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let option = u8::arbitrary(gen) % 4;
            match option {
                0 => ArbitraryAccountStateOp::Add(Value::arbitrary(gen)),
                1 => ArbitraryAccountStateOp::Sub(Value::arbitrary(gen)),
                2 => ArbitraryAccountStateOp::Delegate(PoolId::arbitrary(gen)),
                3 => ArbitraryAccountStateOp::RemoveDelegation,
                _ => panic!("not implemented"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct ArbitraryOperationChain(pub Vec<ArbitraryAccountStateOp>);

    impl Arbitrary for ArbitraryOperationChain {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let len = usize::arbitrary(gen);
            let ops: Vec<ArbitraryAccountStateOp> =
                iter::from_fn(|| Some(ArbitraryAccountStateOp::arbitrary(gen)))
                    .take(len)
                    .collect();
            ArbitraryOperationChain(ops)
        }
    }

    impl ArbitraryOperationChain {
        pub fn get_account_state_after_n_ops(
            &self,
            initial_account_state: AccountState<()>,
            counter: usize,
        ) -> AccountState<()> {
            let n_ops: Vec<ArbitraryAccountStateOp> =
                self.0.iter().cloned().take(counter).collect();
            self.calculate_account_state(initial_account_state, n_ops.iter())
        }

        pub fn get_account_state(
            &self,
            initial_account_state: AccountState<()>,
        ) -> AccountState<()> {
            self.calculate_account_state(initial_account_state, self.0.iter())
        }

        fn calculate_account_state(
            &self,
            initial_account_state: AccountState<()>,
            operations: std::slice::Iter<ArbitraryAccountStateOp>,
        ) -> AccountState<()> {
            let mut spending_strat = initial_account_state.spending.clone();
            let mut delegation = initial_account_state.delegation().clone();
            let mut result_value = initial_account_state.value();

            for operation in operations {
                match operation {
                    ArbitraryAccountStateOp::Add(value) => {
                        result_value = match result_value + *value {
                            Ok(new_value) => new_value,
                            Err(_) => result_value,
                        }
                    }
                    ArbitraryAccountStateOp::Sub(value) => {
                        result_value = match result_value - *value {
                            Ok(new_value) => {
                                spending_strat
                                    .next_verify(spending_strat.get_valid_counter())
                                    .expect("success");
                                new_value
                            }
                            Err(_) => result_value,
                        }
                    }
                    ArbitraryAccountStateOp::Delegate(new_delegation) => {
                        delegation = DelegationType::Full(new_delegation.clone());
                    }
                    ArbitraryAccountStateOp::RemoveDelegation => {
                        delegation = DelegationType::NonDelegated;
                    }
                }
            }
            AccountState {
                spending: spending_strat,
                delegation,
                value: result_value,
                tokens: Hamt::new(),
                last_rewards: LastRewards::default(),
                extra: (),
            }
        }
    }

    impl IntoIterator for ArbitraryOperationChain {
        type Item = ArbitraryAccountStateOp;
        type IntoIter = ::std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    #[quickcheck]
    pub fn account_state_is_consistent(
        mut account_state: AccountState<()>,
        operations: ArbitraryOperationChain,
    ) -> TestResult {
        let initial_account_state = account_state.clone();
        let mut strategy = initial_account_state.spending.clone();
        let mut counter = strategy.get_valid_counter();
        for (op_counter, operation) in operations.clone().into_iter().enumerate() {
            account_state = match operation {
                ArbitraryAccountStateOp::Add(value) => {
                    let should_fail = should_add_fail(account_state.clone(), value);
                    match (should_fail, account_state.add(value)) {
                        (false, Ok(account_state)) => account_state,
                        (true, Err(_)) => account_state,
                        (false,  Err(err)) => return TestResult::error(format!("Operation {}: unexpected add operation failure. Expected success but got: {:?}",op_counter,err)),
                        (true, Ok(account_state)) => return TestResult::error(format!("Operation {}: unexpected add operation success. Expected failure but got: success. AccountState: {:?}",op_counter, &account_state)),
                    }
                }
                ArbitraryAccountStateOp::Sub(value) => {
                    let should_fail = should_sub_fail(account_state.clone(), value);
                    match (should_fail, account_state.sub(counter, value)) {
                        (false, Ok(account_state)) => {
                            strategy.next_verify(counter).expect("success");
                            counter = counter.increment();
                            // check if account has any funds left
                            match account_state {
                                Some(account_state) => account_state,
                                None => return verify_account_lost_all_funds(initial_account_state,operations,op_counter,account_state.unwrap())
                            }
                        }
                        (true, Err(_)) => account_state,
                        (false,  Err(err)) => return TestResult::error(format!("Operation {}: unexpected sub operation failure. Expected success but got: {:?}",op_counter,err)),
                        (true, Ok(account_state)) => return TestResult::error(format!("Operation {}: unexpected sub operation success. Expected failure but got: success. AccountState: {:?}",op_counter, &account_state)),
                    }
                }
                ArbitraryAccountStateOp::Delegate(stake_pool_id) => {
                    account_state.set_delegation(DelegationType::Full(stake_pool_id))
                }
                ArbitraryAccountStateOp::RemoveDelegation => {
                    account_state.set_delegation(DelegationType::NonDelegated)
                }
            };
        }
        let expected_account_state = operations.get_account_state(initial_account_state);
        if expected_account_state == account_state {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Actual AccountState is not equal to expected one. Expected {:?}, but got {:?}",
                expected_account_state, account_state
            ))
        }
    }

    fn verify_account_lost_all_funds(
        initial_account_state: AccountState<()>,
        operations: ArbitraryOperationChain,
        counter: usize,
        actual_account_state: AccountState<()>,
    ) -> TestResult {
        let expected_account =
            operations.get_account_state_after_n_ops(initial_account_state, counter);
        if expected_account.value == Value::zero() {
            TestResult::passed()
        } else {
            TestResult::error(format!("Account is dry out from funds after {} operations, while expectation was different. Expected: {:?}, Actual {:?}",counter,expected_account,actual_account_state))
        }
    }

    fn should_add_fail(account_state: AccountState<()>, value: Value) -> bool {
        (value + account_state.value()).is_err()
    }

    fn should_sub_fail(account_state: AccountState<()>, value: Value) -> bool {
        // should fail if we recieve negative result
        (account_state.value() - value).is_err()
    }

    #[test]
    pub fn delegation_ratio_correct() {
        let fake_pool_id = StakePoolBuilder::new().build().id();
        let parts = 8u8;
        let pools: Vec<(PoolId, u8)> = vec![
            (fake_pool_id.clone(), 2u8),
            (fake_pool_id.clone(), 3u8),
            (fake_pool_id, 3u8),
        ];
        assert!(DelegationRatio::new(parts, pools).is_some());
    }

    #[test]
    pub fn delegation_ratio_zero_parts() {
        let fake_pool_id = StakePoolBuilder::new().build().id();
        let parts = 0u8;
        let pools: Vec<(PoolId, u8)> = vec![
            (fake_pool_id.clone(), 2u8),
            (fake_pool_id.clone(), 3u8),
            (fake_pool_id, 3u8),
        ];
        assert!(DelegationRatio::new(parts, pools).is_none());
    }

    #[test]
    pub fn delegation_ratio_zero_pool_parts() {
        let fake_pool_id = StakePoolBuilder::new().build().id();
        let parts = 8u8;
        let pools: Vec<(PoolId, u8)> = vec![
            (fake_pool_id.clone(), 0u8),
            (fake_pool_id.clone(), 3u8),
            (fake_pool_id, 3u8),
        ];
        assert!(DelegationRatio::new(parts, pools).is_none());
    }

    #[test]
    pub fn delegation_ratio_no_pool_parts() {
        let parts = 1u8;
        let pools: Vec<(PoolId, u8)> = vec![];
        assert!(DelegationRatio::new(parts, pools).is_none());
    }

    #[test]
    pub fn delegation_ratio_pool_parts_larger_than_limit() {
        let fake_pool_id = StakePoolBuilder::new().build().id();
        let parts = (DELEGATION_RATIO_MAX_DECLS + 1) as u8;
        let pools: Vec<(PoolId, u8)> = iter::from_fn(|| Some((fake_pool_id.clone(), 1u8)))
            .take(parts as usize)
            .collect();
        assert!(DelegationRatio::new(parts, pools).is_none());
    }

    #[test]
    pub fn delegation_ratio_different_total() {
        let fake_pool_id = StakePoolBuilder::new().build().id();
        let parts = 8u8;
        let pools: Vec<(PoolId, u8)> = vec![
            (fake_pool_id.clone(), 3u8),
            (fake_pool_id.clone(), 3u8),
            (fake_pool_id, 3u8),
        ];
        assert!(DelegationRatio::new(parts, pools).is_none());
    }

    #[quickcheck]
    pub fn add_rewards(account_state_no_reward: AccountState<()>, value: Value) -> TestResult {
        let initial_value = account_state_no_reward.value();
        let account_state_reward = account_state_no_reward.clone();

        let account_state_no_reward = account_state_no_reward
            .add(value)
            .expect("cannot add value");
        let account_state_reward = account_state_reward
            .add_rewards(1, value)
            .expect("cannot add reward");

        accounts_are_the_same(account_state_no_reward, account_state_reward, initial_value)
    }

    #[quickcheck]
    pub fn new_account_rewards(value: Value) -> TestResult {
        let account_state = AccountState::new(value, ());
        let account_with_reward = AccountState::new_reward(1, value, ());
        accounts_are_the_same(account_state, account_with_reward, Value::zero())
    }

    fn accounts_are_the_same(
        account_without_reward: AccountState<()>,
        account_with_reward: AccountState<()>,
        initial_value: Value,
    ) -> TestResult {
        if account_without_reward.value() != account_with_reward.value() {
            return TestResult::error(format!(
                "value should be the same {} vs {}",
                account_without_reward.value(),
                account_with_reward.value()
            ));
        }

        let expected_reward_account_state =
            (account_with_reward.last_rewards.reward + initial_value).unwrap();
        if account_without_reward.value() != expected_reward_account_state {
            return TestResult::error(format!(
                "reward should be the same {} vs {}",
                account_without_reward.value(),
                expected_reward_account_state
            ));
        }
        TestResult::passed()
    }
}
