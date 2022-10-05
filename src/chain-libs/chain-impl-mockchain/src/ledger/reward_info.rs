use crate::account;
use crate::certificate::PoolId;
use crate::value::Value;
use std::collections::BTreeMap;
use std::default::Default;

/// Control what information will be extracted from the rewarding distribution process
///
/// By default, only the really cheap basic information will be added,
/// but for specific it's possible to get extra information that cost.
#[derive(Default, Debug, Clone, Copy)]
pub struct RewardsInfoParameters {
    report_stake_pools: bool,
    report_accounts: bool,
}

impl RewardsInfoParameters {
    pub fn report_all() -> Self {
        RewardsInfoParameters {
            report_stake_pools: true,
            report_accounts: true,
        }
    }
}

/// The epoch reward information.
///
/// note that stake_pools and accounts are
/// only filled up if the reward info parameters
/// report_stake_pools and report_accounts (respectively)
/// are turned on.
#[derive(Debug, Clone)]
pub struct EpochRewardsInfo {
    /// Params used
    pub params: RewardsInfoParameters,
    /// Total Drawn from reward escrow pot for this epoch
    pub drawn: Value,
    /// Fees contributed into the pot this epoch
    pub fees: Value,
    /// Value added to the treasury
    pub treasury: Value,
    /// Amount added to each pool id. structure can be empty.
    pub stake_pools: BTreeMap<PoolId, (Value, Value)>,
    /// Amount added to each account. structure can be empty.
    pub accounts: BTreeMap<account::Identifier, Value>,
}

impl EpochRewardsInfo {
    pub fn new(params: RewardsInfoParameters) -> EpochRewardsInfo {
        EpochRewardsInfo {
            params,
            drawn: Value::zero(),
            fees: Value::zero(),
            treasury: Value::zero(),
            stake_pools: BTreeMap::new(),
            accounts: BTreeMap::new(),
        }
    }

    pub fn set_treasury(&mut self, value: Value) {
        // mostly prevent this to be set twice (unless it is set zero)
        assert_eq!(self.treasury, Value::zero());
        self.treasury = value;
    }

    pub fn set_stake_pool(&mut self, pool: &PoolId, owned: Value, distributed: Value) {
        if self.params.report_stake_pools {
            self.stake_pools.insert(pool.clone(), (owned, distributed));
        }
    }

    pub fn add_to_account(&mut self, account: &account::Identifier, value: Value) {
        if self.params.report_accounts {
            let ent = self.accounts.entry(account.clone()).or_default();
            *ent = (*ent + value).unwrap()
        }
    }

    pub fn total(&self) -> Value {
        (self.drawn + self.fees).unwrap()
    }
}
