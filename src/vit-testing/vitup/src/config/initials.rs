use chain_addr::Discrimination;
use chain_impl_mockchain::value::Value;
use jormungandr_testing_utils::testing::network::{ExternalWalletTemplate, WalletTemplate};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initials(pub Vec<Initial>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    AboveThreshold {
        above_threshold: usize,
        pin: String,
    },
    BelowThreshold {
        below_threshold: usize,
        pin: String,
    },
    AroundLevel {
        count: usize,
        level: u64,
        pin: String,
    },
    ZeroFunds {
        zero_funds: usize,
        pin: String,
    },
    Wallet {
        name: String,
        funds: usize,
        pin: String,
    },
    External {
        address: String,
        funds: u64,
    },
}

pub const GRACE_VALUE: u64 = 1;

impl Default for Initials {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl Initials {
    pub fn extend(&mut self, extend: Self) {
        for element in extend.0.into_iter() {
            self.0.push(element);
        }
    }

    pub fn any(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn extend_from_external(&mut self, initials: Vec<jormungandr_lib::interfaces::Initial>) {
        self.extend(Self::new_from_external(initials))
    }

    pub fn zero_funds_count(&self) -> usize {
        for initial in self.0.iter() {
            if let Initial::ZeroFunds { zero_funds, pin: _ } = initial {
                return *zero_funds;
            }
        }
        0
    }

    pub fn count(&self) -> usize {
        let mut sum = 0;
        for initial in self.0.iter() {
            match initial {
                Initial::ZeroFunds { zero_funds, pin: _ } => sum += *zero_funds,
                Initial::BelowThreshold {
                    below_threshold,
                    pin: _,
                } => sum += below_threshold,
                Initial::AboveThreshold {
                    above_threshold,
                    pin: _,
                } => sum += above_threshold,
                Initial::Wallet { .. } => sum += 1,
                Initial::AroundLevel {
                    level: _,
                    count,
                    pin: _,
                } => sum += count,
                _ => {}
            }
        }
        sum
    }

    pub fn zero_funds_pin(&self) -> Option<String> {
        for initial in self.0.iter() {
            if let Initial::ZeroFunds { zero_funds: _, pin } = initial {
                return Some(pin.clone());
            }
        }
        None
    }

    pub fn new_above_threshold(count: usize, pin: &str) -> Self {
        Self(vec![Initial::AboveThreshold {
            above_threshold: count,
            pin: pin.to_string(),
        }])
    }

    pub fn new_from_external(initials: Vec<jormungandr_lib::interfaces::Initial>) -> Self {
        let mut templates = Vec::new();
        for initial in initials.iter() {
            if let jormungandr_lib::interfaces::Initial::Fund(initial_utxos) = initial {
                for utxo in initial_utxos.iter() {
                    templates.push(Initial::External {
                        address: utxo.address.to_string(),
                        funds: utxo.value.into(),
                    });
                }
            }
        }
        Self(templates)
    }

    pub fn external_templates(&self) -> Vec<ExternalWalletTemplate> {
        let mut templates = Vec::new();
        for (index, initial) in self.0.iter().enumerate() {
            if let Initial::External { funds, address } = initial {
                templates.push(ExternalWalletTemplate::new(
                    format!("wallet_{}", index + 1),
                    Value(*funds as u64),
                    address.to_string(),
                ));
            }
        }
        templates
    }

    pub fn templates(
        &self,
        threshold: u64,
        discrimination: Discrimination,
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
                } => {
                    for _ in 0..*above_threshold {
                        above_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_above_{}", above_threshold_index, threshold);
                        let value: u64 = rand.gen_range(GRACE_VALUE..=threshold - GRACE_VALUE);
                        templates.insert(
                            WalletTemplate::new_account(
                                wallet_alias,
                                Value(threshold + value),
                                discrimination,
                            ),
                            pin.to_string(),
                        );
                    }
                }
                Initial::BelowThreshold {
                    below_threshold,
                    pin,
                } => {
                    for _ in 0..*below_threshold {
                        below_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_below_{}", below_threshold_index, threshold);
                        let value: u64 = rand.gen_range(GRACE_VALUE..=threshold - GRACE_VALUE);
                        templates.insert(
                            WalletTemplate::new_account(
                                wallet_alias,
                                Value(threshold - value),
                                discrimination,
                            ),
                            pin.to_string(),
                        );
                    }
                }
                Initial::Wallet { name, funds, pin } => {
                    let wallet_alias = format!("wallet_{}", name);
                    templates.insert(
                        WalletTemplate::new_account(
                            wallet_alias,
                            Value(*funds as u64),
                            discrimination,
                        ),
                        pin.to_string(),
                    );
                }
                Initial::AroundLevel { level, count, pin } => {
                    for _ in 0..*count {
                        around_level_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_around_{}", around_level_index, threshold);
                        let value: u64 = rand.gen_range(level - GRACE_VALUE..=level + GRACE_VALUE);
                        templates.insert(
                            WalletTemplate::new_account(wallet_alias, Value(value), discrimination),
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
