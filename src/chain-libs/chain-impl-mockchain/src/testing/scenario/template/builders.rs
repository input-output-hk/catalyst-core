use crate::{
    rewards::{Ratio, TaxType},
    testing::data::Wallet,
    testing::scenario::{scenario_builder::ScenarioBuilderError, template::StakePoolDef},
    value::Value,
};

use super::{StakePoolTemplate, WalletTemplate};

use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

#[derive(Clone, Debug)]
pub struct WalletTemplateBuilder {
    alias: String,
    delagate_alias: Option<String>,
    ownership_alias: Option<String>,
    initial_value: Option<Value>,
}

impl WalletTemplateBuilder {
    pub fn new(alias: &str) -> Self {
        WalletTemplateBuilder {
            alias: alias.to_owned(),
            delagate_alias: None,
            ownership_alias: None,
            initial_value: None,
        }
    }

    pub fn with(&mut self, value: u64) -> &mut Self {
        self.initial_value = Some(Value(value));
        self
    }

    pub fn owns(&mut self, ownership_alias: &str) -> &mut Self {
        self.ownership_alias = Some(ownership_alias.to_owned());
        self
    }

    pub fn delegates_to(&mut self, delegates_to_alias: &str) -> &mut Self {
        self.delagate_alias = Some(delegates_to_alias.to_owned());
        self
    }

    pub fn owns_and_delegates_to(&mut self, ownership_alias: &str) -> &mut Self {
        self.owns(ownership_alias).delegates_to(ownership_alias);
        self
    }

    pub fn build(&self) -> Result<WalletTemplate, ScenarioBuilderError> {
        let value = self
            .initial_value
            .ok_or(ScenarioBuilderError::UndefinedValueForWallet {
                alias: self.alias.clone(),
            })?;

        Ok(WalletTemplate {
            alias: self.alias.clone(),
            stake_pool_delegate_alias: self.delagate_alias.clone(),
            stake_pool_owner_alias: self.ownership_alias.clone(),
            initial_value: value,
        })
    }
}

pub struct StakePoolTemplateBuilder {
    ownership_map: HashMap<String, HashSet<WalletTemplate>>,
    delegation_map: HashMap<String, HashSet<WalletTemplate>>,
}

impl StakePoolTemplateBuilder {
    pub fn new(initials: &[WalletTemplate]) -> Self {
        StakePoolTemplateBuilder {
            ownership_map: Self::build_ownersip_map(initials),
            delegation_map: Self::build_delegation_map(initials),
        }
    }

    pub fn build_stake_pool_templates(
        &self,
        wallets: Vec<Wallet>,
    ) -> Result<Vec<StakePoolTemplate>, ScenarioBuilderError> {
        self.defined_stake_pools_aliases()
            .iter()
            .map(|stake_pool_alias| {
                let owners = self.ownership_map.get(stake_pool_alias).ok_or(
                    ScenarioBuilderError::NoOwnersForStakePool {
                        alias: stake_pool_alias.to_string(),
                    },
                )?;

                let owners_public_keys = wallets
                    .iter()
                    .filter(|w| owners.iter().any(|u| u.alias() == w.alias()))
                    .map(|w| w.public_key())
                    .collect();

                Ok(StakePoolTemplate {
                    alias: stake_pool_alias.to_string(),
                    owners: owners_public_keys,
                })
            })
            .collect()
    }

    pub fn defined_stake_pools_aliases(&self) -> HashSet<String> {
        self.ownership_map
            .clone()
            .into_iter()
            .chain(self.delegation_map.clone())
            .map(|(k, _)| k)
            .collect()
    }

    fn build_ownersip_map(
        initials: &[WalletTemplate],
    ) -> HashMap<String, HashSet<WalletTemplate>> {
        let mut output: HashMap<String, HashSet<WalletTemplate>> = HashMap::new();
        for wallet_template in initials.iter().filter(|w| w.owns_stake_pool().is_some()) {
            let delegate_alias = wallet_template.owns_stake_pool().unwrap();

            output
                .entry(delegate_alias)
                .or_default()
                .insert(
                    wallet_template.clone()
                );
        }
        output
    }

    fn build_delegation_map(
        initials: &[WalletTemplate],
    ) -> HashMap<String, HashSet<WalletTemplate>> {
        let mut output: HashMap<String, HashSet<WalletTemplate>> = HashMap::new();
        for wallet_template in initials
            .iter()
            .filter(|w| w.delegates_stake_pool().is_some())
        {
            let stake_pool_alias = wallet_template.delegates_stake_pool().unwrap();

            output
                .entry(stake_pool_alias)
                .or_default()
                .insert(
                    wallet_template.clone()
                );
        }
        output
    }
}

#[derive(Clone, Debug)]
pub struct StakePoolDefBuilder {
    alias: String,
    permissions_threshold: u8,
    reward_account: bool,
    tax_type: Option<TaxType>,
}

impl StakePoolDefBuilder {
    pub fn new(alias: &str) -> Self {
        StakePoolDefBuilder {
            alias: alias.to_owned(),
            permissions_threshold: 1u8,
            reward_account: false,
            tax_type: None,
        }
    }

    pub fn with_permissions_threshold(&mut self, threshold: u8) -> &mut Self {
        self.permissions_threshold = threshold;
        self
    }

    pub fn with_reward_account(&mut self, reward_account: bool) -> &mut Self {
        self.reward_account = reward_account;
        self
    }

    pub fn tax_ratio(&mut self, numerator: u64, denominator: u64) -> &mut Self {
        self.tax_type = Some(TaxType {
            fixed: Value(0),
            ratio: Ratio {
                numerator: numerator,
                denominator: NonZeroU64::new(denominator).unwrap(),
            },
            max_limit: None,
        });
        self
    }

    pub fn tax_limit(&mut self, limit: u64) -> &mut Self {
        match self.tax_type.as_mut() {
            Some(tax_type) => tax_type.max_limit = Some(NonZeroU64::new(limit).unwrap()),
            None => unreachable!("setting tax limit for none TaxType"),
        };
        self
    }

    pub fn fixed_tax(&mut self, value: u64) -> &mut Self {
        self.tax_type = Some(TaxType {
            fixed: Value(value),
            ratio: Ratio::zero(),
            max_limit: None,
        });
        self
    }

    pub fn no_tax(&mut self) -> &mut Self {
        self.tax_type = Some(TaxType::zero());
        self
    }

    pub fn build(&self) -> StakePoolDef {
        StakePoolDef {
            alias: self.alias.clone(),
            permissions_threshold: Some(self.permissions_threshold),
            has_reward_account: self.reward_account,
            tax_type: self.tax_type,
        }
    }
}
