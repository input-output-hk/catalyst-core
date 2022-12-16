use crate::config::initials::Role;
use chain_addr::AddressReadable;
use chain_addr::Discrimination;
use chain_impl_mockchain::value::Value;
use fake::faker::name::en::Name;
use fake::Fake;
use hersir::config::{WalletTemplate, WalletTemplateBuilder};
use jormungandr_lib::interfaces::InitialUTxO;
use jormungandr_lib::interfaces::TokenIdentifier;
use rand::Rng;
use serde::{Deserialize, Serialize};
use snapshot_lib::VoterHIR;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initials(pub Vec<Initial>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    AboveThreshold {
        above_threshold: usize,
        pin: String,
        #[serde(default)]
        role: Role,
    },
    BelowThreshold {
        below_threshold: usize,
        pin: String,
        #[serde(default)]
        role: Role,
    },
    AroundLevel {
        count: usize,
        level: u64,
        pin: String,
        #[serde(default)]
        role: Role,
    },
    ZeroFunds {
        zero_funds: usize,
        pin: String,
        #[serde(default)]
        role: Role,
    },
    Wallet {
        name: String,
        funds: u64,
        pin: String,
        #[serde(default)]
        role: Role,
    },
    External {
        address: String,
        funds: u64,
        #[serde(default)]
        role: Role,
    },
}

impl Initial {
    pub fn new_random_wallet(funds: u64) -> Self {
        Self::Wallet {
            name: Name().fake::<String>(),
            funds,
            pin: "1234".to_string(),
            role: Default::default(),
        }
    }
}
pub const GRACE_VALUE: u64 = 1;

// suppress, because when implementing complier gives error: deriving `Default` on enums is experimental
#[allow(clippy::derivable_impls)]
impl Default for Initials {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[allow(dead_code)]
pub fn convert_to_external_utxo(initials: Vec<InitialUTxO>) -> Vec<Initial> {
    initials
        .into_iter()
        .map(|utxo| Initial::External {
            address: utxo.address.to_string(),
            funds: utxo.value.into(),
            role: Default::default(),
        })
        .collect()
}

impl Initials {
    pub fn extend(&mut self, extend: Self) {
        for element in extend.0.into_iter() {
            self.0.push(element);
        }
    }

    pub fn push(&mut self, initial: Initial) {
        self.0.push(initial);
    }

    pub fn any(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn extend_from_external(
        &mut self,
        initials: Vec<VoterHIR>,
        discrimination: Discrimination,
    ) {
        self.extend(Self::new_from_external(initials, discrimination))
    }

    pub fn zero_funds_count(&self) -> usize {
        for initial in self.0.iter() {
            if let Initial::ZeroFunds {
                zero_funds,
                pin: _,
                role: _,
            } = initial
            {
                return *zero_funds;
            }
        }
        0
    }

    pub fn count(&self) -> usize {
        let mut sum = 0;
        for initial in self.0.iter() {
            match initial {
                Initial::ZeroFunds {
                    zero_funds,
                    pin: _,
                    role: _,
                } => sum += *zero_funds,
                Initial::BelowThreshold {
                    below_threshold,
                    pin: _,
                    role: _,
                } => sum += below_threshold,
                Initial::AboveThreshold {
                    above_threshold,
                    pin: _,
                    role: _,
                } => sum += above_threshold,
                Initial::Wallet { .. } => sum += 1,
                Initial::AroundLevel {
                    level: _,
                    count,
                    pin: _,
                    role: _,
                } => sum += count,
                _ => {}
            }
        }
        sum
    }

    pub fn zero_funds_pin(&self) -> Option<String> {
        for initial in self.0.iter() {
            if let Initial::ZeroFunds {
                zero_funds: _,
                pin,
                role: _,
            } = initial
            {
                return Some(pin.clone());
            }
        }
        None
    }

    pub fn new_above_threshold(count: usize, pin: &str) -> Self {
        Self(vec![Initial::AboveThreshold {
            above_threshold: count,
            pin: pin.to_string(),
            role: Default::default(),
        }])
    }

    pub fn new_from_external(initials: Vec<VoterHIR>, discrimination: Discrimination) -> Self {
        let prefix = match discrimination {
            Discrimination::Production => "ca".to_string(),
            Discrimination::Test => "ta".to_string(),
        };

        let mut templates = Vec::new();
        for initial in initials.iter() {
            templates.push(Initial::External {
                address: AddressReadable::from_address(
                    &prefix,
                    &initial.voting_key.to_address(discrimination),
                )
                .to_string(),
                funds: initial.voting_power.into(),
                role: Role::from_str(&initial.voting_group).unwrap(),
            });
        }
        Self(templates)
    }

    pub fn external_templates(
        &self,
        roles: impl Fn(&Role) -> TokenIdentifier,
    ) -> Vec<WalletTemplate> {
        let mut templates = Vec::new();
        for initial in self.0.iter() {
            if let Initial::External {
                funds,
                address,
                role,
            } = initial
            {
                let funds = *funds;

                let mut tokens = HashMap::new();
                tokens.insert(roles(role), funds);

                templates.push(WalletTemplate::new_external(
                    address.to_string(),
                    Value(funds),
                    tokens,
                ));
            }
        }
        templates
    }

    pub fn templates(
        &self,
        threshold: u64,
        discrimination: Discrimination,
        roles: impl Fn(&Role) -> TokenIdentifier,
    ) -> HashMap<WalletTemplate, String> {
        let mut rand = rand::thread_rng();
        let mut above_threshold_index = 0;
        let mut below_threshold_index = 0;
        let mut around_level_index = 0;
        let mut templates = HashMap::new();

        for initial in self.0.iter() {
            match initial {
                Initial::AboveThreshold {
                    above_threshold,
                    pin,
                    role,
                } => {
                    for _ in 0..*above_threshold {
                        above_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_above_{}", above_threshold_index, threshold);
                        let value: u64 =
                            threshold + rand.gen_range(GRACE_VALUE..=threshold - GRACE_VALUE);
                        templates.insert(
                            WalletTemplateBuilder::new(wallet_alias)
                                .with(value)
                                .with_token(roles(role), value)
                                .discrimination(discrimination)
                                .build(),
                            pin.to_string(),
                        );
                    }
                }
                Initial::BelowThreshold {
                    below_threshold,
                    pin,
                    role,
                } => {
                    for _ in 0..*below_threshold {
                        below_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_below_{}", below_threshold_index, threshold);
                        let value: u64 =
                            threshold - rand.gen_range(GRACE_VALUE..=threshold - GRACE_VALUE);
                        templates.insert(
                            WalletTemplateBuilder::new(wallet_alias)
                                .with(value)
                                .with_token(roles(role), value)
                                .discrimination(discrimination)
                                .build(),
                            pin.to_string(),
                        );
                    }
                }
                Initial::Wallet {
                    name,
                    funds,
                    pin,
                    role,
                } => {
                    templates.insert(
                        WalletTemplateBuilder::new(name)
                            .with(*funds)
                            .discrimination(discrimination)
                            .with_token(roles(role), *funds)
                            .build(),
                        pin.to_string(),
                    );
                }
                Initial::AroundLevel {
                    level,
                    count,
                    pin,
                    role,
                } => {
                    for _ in 0..*count {
                        around_level_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_around_{}", around_level_index, threshold);
                        let value: u64 = rand.gen_range(level - GRACE_VALUE..=level + GRACE_VALUE);
                        templates.insert(
                            WalletTemplateBuilder::new(wallet_alias)
                                .with(value)
                                .discrimination(discrimination)
                                .with_token(roles(role), value)
                                .build(),
                            pin.to_string(),
                        );
                    }
                }
                _ => {
                    //skip
                }
            }
        }
        templates
    }
}
