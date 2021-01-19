use chain_impl_mockchain::value::Value;
use jormungandr_testing_utils::testing::network_builder::WalletTemplate;
use rand::Rng;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initials(pub Vec<Initial>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    AboveThreshold { above_threshold: usize },
    BelowThreshold { below_threshold: usize },
    ZeroFunds { zero_funds: usize },
    Wallet { name: String, funds: usize },
}

pub const GRACE_VALUE: u64 = 100;

impl Default for Initials {
    fn default() -> Self {
        let initials: Vec<Initial> = std::iter::from_fn(|| {
            Some(Initial::AboveThreshold {
                above_threshold: 10,
            })
        })
        .take(1)
        .collect();
        Self(initials)
    }
}

impl Initials {
    pub fn zero_funds_count(&self) -> usize {
        for initial in self.0.iter() {
            if let Initial::ZeroFunds { zero_funds } = initial {
                return *zero_funds;
            }
        }
        0
    }

    pub fn new_above_threshold(count: usize) -> Initials {
        let initials: Vec<Initial> = std::iter::from_fn(|| {
            Some(Initial::AboveThreshold {
                above_threshold: count,
            })
        })
        .take(count)
        .collect();
        Self(initials)
    }

    pub fn templates(&self, threshold: u64) -> Vec<WalletTemplate> {
        let mut rand = rand::thread_rng();
        let mut above_threshold_index = 0;
        let mut below_threshold_index = 0;

        let mut templates = Vec::new();

        for initial in self.0.iter() {
            match initial {
                Initial::AboveThreshold { above_threshold } => {
                    for _ in 0..*above_threshold {
                        above_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_above_{}", above_threshold_index, threshold);
                        let value: u64 = rand.gen_range(GRACE_VALUE, threshold - GRACE_VALUE);
                        templates.push(WalletTemplate::new_utxo(
                            wallet_alias,
                            Value(threshold + value),
                        ));
                    }
                }
                Initial::BelowThreshold { below_threshold } => {
                    for _ in 0..*below_threshold {
                        below_threshold_index += 1;
                        let wallet_alias =
                            format!("wallet_{}_below_{}", below_threshold_index, threshold);
                        let value: u64 = rand.gen_range(GRACE_VALUE, threshold - GRACE_VALUE);
                        templates.push(WalletTemplate::new_utxo(
                            wallet_alias,
                            Value(threshold - value),
                        ));
                    }
                }
                Initial::ZeroFunds { zero_funds: _ } => {
                    //skip
                }
                Initial::Wallet { name, funds } => {
                    let wallet_alias = format!("wallet_{}", name);
                    templates.push(WalletTemplate::new_utxo(wallet_alias, Value(*funds as u64)));
                }
            }
        }
        templates
    }
}

#[cfg(test)]
mod tests {
    use super::Initials;
    use jortestkit;

    #[test]
    pub fn test() {
        let data = jortestkit::file::read_file("C:\\tmp\\text.json");
        let json: Initials = serde_json::from_str(&data).unwrap();
        println!("{:?}", json)
    }
}
