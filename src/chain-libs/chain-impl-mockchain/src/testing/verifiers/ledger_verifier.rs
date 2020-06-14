use crate::{
    account::{Identifier, Ledger as AccountLedger},
    accounting::account::{account_state::AccountState, DelegationType},
    certificate::{PoolId, PoolRegistration},
    ledger::{ledger::Ledger, Pots},
    stake::PoolsState,
    stake::{Stake, StakeDistribution},
    testing::data::{AddressData, StakePool},
    utxo,
    value::Value,
};
use chain_addr::Address;
use chain_crypto::{Ed25519, PublicKey};
use std::fmt;

#[derive(Clone)]
pub struct Info {
    info: Option<String>,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.info {
            Some(info) => write!(f, "{}", info),
            None => write!(f, ""),
        }
    }
}

impl Info {
    pub fn from_str<S: Into<String>>(info: S) -> Self {
        Info {
            info: Some(info.into()),
        }
    }

    pub fn empty() -> Self {
        Info { info: None }
    }
}

pub struct LedgerStateVerifier {
    ledger: Ledger,
    info: Info,
}

impl LedgerStateVerifier {
    pub fn new(ledger: Ledger) -> Self {
        LedgerStateVerifier {
            ledger,
            info: Info::empty(),
        }
    }

    pub fn info<S: Into<String>>(&mut self, info: S) -> &mut Self {
        self.info = Info::from_str(info);
        self
    }

    pub fn utxo_contains(&self, entry: &utxo::Entry<Address>) -> &Self {
        assert_eq!(
            self.ledger.utxos.iter().find(|x| *x == entry.clone()),
            Some(entry.clone())
        );
        self
    }

    pub fn and(&self) -> &Self {
        self
    }

    pub fn accounts_contains(
        &self,
        id: Identifier,
        expected_account_state: AccountState<()>,
    ) -> &Self {
        let account_state = self.ledger.accounts.get_state(&id).unwrap();
        assert_eq!(account_state.clone(), expected_account_state);
        self
    }

    pub fn account(&self, address_data: AddressData) -> AccountVerifier {
        AccountVerifier::new(
            self.ledger.accounts.clone(),
            address_data,
            self.info.clone(),
        )
    }

    pub fn utxos_count_is(&self, count: usize) -> &Self {
        assert_eq!(
            self.ledger.utxos.iter().count(),
            count,
            "Utxo count should be equal to {:?} {}",
            count,
            self.info
        );
        self
    }

    pub fn accounts_count_is(&self, count: usize) -> &Self {
        assert_eq!(
            self.ledger.accounts.iter().count(),
            count,
            "Utxo count should be equal to {:?} {}",
            count,
            self.info
        );
        self
    }

    pub fn multisigs_count_is_zero(&self) -> &Self {
        assert_eq!(self.ledger.multisig.iter_accounts().count(), 0);
        assert_eq!(self.ledger.multisig.iter_declarations().count(), 0);
        self
    }

    pub fn distribution(&self) -> DistributionVerifier {
        DistributionVerifier::new(self.ledger.get_stake_distribution(), self.info.clone())
    }

    pub fn stake_pools(&self) -> StakePoolsVerifier {
        StakePoolsVerifier::new(self.ledger.delegation.clone(), self.info.clone())
    }

    pub fn stake_pool(&self, stake_pool_id: &PoolId) -> StakePoolVerifier {
        let stake_pool_reg = self
            .ledger
            .delegation
            .lookup_reg(stake_pool_id)
            .expect("stake pool does not exists");
        StakePoolVerifier::new(
            stake_pool_id.clone(),
            stake_pool_reg.clone(),
            self.info.clone(),
        )
    }

    pub fn total_value_is(&self, value: &Value) -> &Self {
        let actual_value = self.ledger.get_total_value().expect("total amount too big");
        assert_eq!(
            *value, actual_value,
            "Expected value {:?} vs {:?} of actual {}",
            *value, actual_value, self.info
        );
        self
    }

    // Does not cover situation in which we have two identical utxos
    pub fn address_has_expected_balance(&self, address: AddressData, value: Value) -> &Self {
        if self.ledger.accounts.exists(&address.to_id()) {
            self.account_has_expected_balance(address, value)
        } else {
            self.utxo_has_expected_balance(address, value)
        }
    }

    pub fn account_has_expected_balance(&self, address: AddressData, value: Value) -> &Self {
        let account_state = self
            .ledger
            .accounts
            .get_state(&address.to_id())
            .expect("account does not exists while it should");
        assert_eq!(account_state.value(), value);
        self
    }

    pub fn utxo_has_expected_balance(&self, address_data: AddressData, value: Value) -> &Self {
        let utxo = self
            .ledger
            .utxos
            .iter()
            .find(|x| *x.output == address_data.make_output(value));
        if value == Value::zero() {
            assert!(utxo.is_none());
            self
        } else {
            let utxo = utxo.unwrap();
            assert_eq!(utxo.output.value, value);
            self
        }
    }

    pub fn pots(&self) -> PotsVerifier {
        PotsVerifier::new(self.ledger.pots.clone(), self.info.clone())
    }
}

pub struct AccountVerifier {
    accounts: AccountLedger,
    address: AddressData,
    info: Info,
}

impl AccountVerifier {
    pub fn new(accounts: AccountLedger, address: AddressData, info: Info) -> Self {
        AccountVerifier {
            accounts,
            address,
            info,
        }
    }

    pub fn has_last_reward(&self, value: &Value) -> &Self {
        let reward_value = self
            .accounts
            .get_state(&self.address.to_id())
            .expect("cannot find account")
            .last_rewards
            .reward;
        let expected_value = *value;
        assert_eq!(
            reward_value, expected_value,
            "incorrect rewards value {} vs {} {}",
            reward_value, expected_value, self.info
        );
        self
    }

    pub fn and(&self) -> &Self {
        self
    }

    pub fn does_not_exist(&self) -> &Self {
        assert!(
            !self.accounts.exists(&self.address.to_id()),
            "account should not exists {}",
            self.info
        );
        self
    }

    pub fn delegation(&self) -> DelegationVerifier {
        let account_state = self
            .accounts
            .get_state(&self.address.to_id())
            .expect("account does not exists");
        DelegationVerifier::new(account_state.delegation().clone(), self.info.clone())
    }

    pub fn has_value(&self, value: &Value) -> &Self {
        let actual_value = self
            .accounts
            .get_state(&self.address.to_id())
            .expect("cannot find account")
            .value;
        let expected_value = *value;
        assert_eq!(
            actual_value, expected_value,
            "incorrect account value {} vs {} {}",
            actual_value, expected_value, self.info
        );
        self
    }
}

pub struct DelegationVerifier {
    delegation_type: DelegationType,
    info: Info,
}

impl DelegationVerifier {
    pub fn new(delegation_type: DelegationType, info: Info) -> Self {
        Self {
            delegation_type,
            info,
        }
    }

    pub fn is_fully_delegated_to(&self, expected_pool_id: PoolId) -> &Self {
        match &self.delegation_type {
            DelegationType::NonDelegated => panic!(format!(
                "{}: wrong wrong delegation type NonDelegated, Expected: Full",
                self.info
            )),
            DelegationType::Full(pool_id) => assert_eq!(
                *pool_id, expected_pool_id,
                "{}: wrong pool id. Expected: {}, but got: {}",
                self.info, expected_pool_id, pool_id
            ),
            DelegationType::Ratio(_) => panic!(format!(
                "{}: wrong delegation type: Ratio, , Expected: Full",
                self.info
            )),
        };
        self
    }
}

pub struct PotsVerifier {
    pots: Pots,
    info: Info,
}

impl PotsVerifier {
    pub fn new(pots: Pots, info: Info) -> Self {
        PotsVerifier { pots, info }
    }

    pub fn has_fee_equals_to(&self, value: &Value) -> &Self {
        assert_eq!(
            self.pots.fees, *value,
            "incorrect pot fee value {}",
            self.info
        );
        self
    }

    pub fn and(&self) -> &Self {
        self
    }

    pub fn has_treasury_equals_to(&self, value: &Value) -> &Self {
        assert_eq!(
            self.pots.treasury.value(),
            *value,
            "incorrect treasury value {}",
            self.info
        );
        self
    }

    pub fn has_remaining_rewards_equals_to(&self, value: &Value) -> &Self {
        assert_eq!(
            self.pots.rewards, *value,
            "incorrect remaining rewards value {}",
            self.info
        );
        self
    }
}

pub struct StakePoolsVerifier {
    delegation: PoolsState,
    info: Info,
}

impl StakePoolsVerifier {
    pub fn new(delegation: PoolsState, info: Info) -> Self {
        StakePoolsVerifier { delegation, info }
    }

    pub fn is_retired(&self, stake_pool: &StakePool) {
        assert!(
            !self.delegation.stake_pool_exists(&stake_pool.id()),
            "stake pool {} should be retired ({}), but it is not",
            stake_pool.alias(),
            self.info
        );
    }

    pub fn is_not_retired(&self, stake_pool: &StakePool) {
        assert!(
            self.delegation.stake_pool_exists(&stake_pool.id()),
            "stake pool {} should be active ({}), but it is retired",
            stake_pool.alias(),
            self.info
        );
    }
}

pub struct StakePoolVerifier {
    stake_pool_id: PoolId,
    stake_pool_reg: PoolRegistration,
    info: Info,
}

impl StakePoolVerifier {
    pub fn new(stake_pool_id: PoolId, stake_pool_reg: PoolRegistration, info: Info) -> Self {
        Self {
            stake_pool_id,
            stake_pool_reg,
            info,
        }
    }

    pub fn serial_eq(&self, serial: u128) -> &Self {
        assert_eq!(
            self.stake_pool_reg.serial, serial,
            "{}: stake pool ({}) has incorrect serial, expected '{}', but got '{}'",
            self.info, self.stake_pool_id, serial, self.stake_pool_reg.serial
        );
        self
    }

    pub fn owners_eq(&self, public_keys: Vec<PublicKey<Ed25519>>) -> &Self {
        assert_eq!(
            self.stake_pool_reg.owners, public_keys,
            "{}: stake pool ({}) has incorrect owners list, expected '{:?}', but got '{:?}'",
            self.info, self.stake_pool_id, public_keys, self.stake_pool_reg.owners
        );
        self
    }
}

pub struct DistributionVerifier {
    stake_distribution: StakeDistribution,
    info: Info,
}

impl DistributionVerifier {
    pub fn new(stake_distribution: StakeDistribution, info: Info) -> Self {
        DistributionVerifier {
            stake_distribution,
            info,
        }
    }

    pub fn dangling_is(&self, dangling: Stake) -> &Self {
        assert_eq!(
            dangling, self.stake_distribution.dangling,
            "wrong unassigned distribution value {}",
            self.info
        );
        self
    }

    pub fn and(&self) -> &Self {
        self
    }

    pub fn unassigned_is(&self, unassigned: Stake) -> &Self {
        assert_eq!(
            unassigned, self.stake_distribution.unassigned,
            "wrong unassigned distribution value {}",
            self.info
        );
        self
    }

    pub fn pools_total_stake_is(&self, pools_total: Stake) -> &Self {
        assert_eq!(
            pools_total,
            self.stake_distribution.total_stake(),
            "wrong total stake {}",
            self.info
        );
        self
    }

    pub fn pools_distribution_is(&self, expected_distribution: Vec<(PoolId, Value)>) -> &Self {
        for (pool_id, value) in expected_distribution {
            let stake = self.stake_distribution.get_stake_for(&pool_id);
            assert!(
                stake.is_some(),
                "pool with id {:?} does not exist {}",
                pool_id,
                self.info
            );
            let stake = stake.unwrap();
            assert_eq!(
                stake,
                Stake::from_value(value),
                "wrong total stake for pool with id {} {}",
                pool_id,
                self.info
            );
        }
        self
    }
}
