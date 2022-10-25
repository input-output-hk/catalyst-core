use crate::config::initials::Role;
use chain_crypto::PublicKeyFromStrError;
use hersir::builder::Wallet as WalletSettings;
use jormungandr_lib::crypto::account::Identifier;
use serde::{Deserialize, Serialize};
use snapshot_lib::VoterHIR;
use std::str::FromStr;
use thor::{Wallet, WalletAlias};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initials {
    pub tag: String,
    pub content: Vec<Initial>,
}

impl Default for Initials {
    fn default() -> Self {
        Self {
            tag: "".to_string(),
            content: Vec::new(),
        }
    }
}

impl Initials {
    pub fn from_voters_hir(voters_hir: Vec<VoterHIR>, tag: String) -> Self {
        Self {
            tag,
            content: voters_hir
                .iter()
                .map(|hir| Initial::External {
                    key: hir.voting_key.to_hex(),
                    funds: hir.voting_power.into(),
                    role: Role::from_str(&hir.voting_group).unwrap(),
                })
                .collect(),
        }
    }

    pub fn as_voters_hirs(
        &self,
        defined_wallets: Vec<(WalletAlias, &WalletSettings)>,
    ) -> Result<Vec<VoterHIR>, Error> {
        let mut voter_hirs = Vec::new();

        for initial in self.content.iter() {
            match initial {
                Initial::Random { count, level, role } => {
                    for _ in 0..*count {
                        voter_hirs.push(VoterHIR {
                            voting_key: Wallet::default().account_id(),
                            voting_group: role.to_string(),
                            voting_power: (*level).into(),
                        });
                    }
                }
                Initial::Wallet { name, funds, role } => {
                    let wallet = defined_wallets
                        .iter()
                        .cloned()
                        .find(|(x, _)| {
                            *x.to_lowercase() == name.to_lowercase()
                        })
                        .map(|(_, y)| y.clone())
                        .ok_or_else(|| Error::CannotFindAlias(name.to_string()))?;
                    voter_hirs.push(VoterHIR {
                        voting_power: (*funds).into(),
                        voting_key: Identifier::from(wallet.identifier()),
                        voting_group: role.to_string(),
                    });
                }
                Initial::WalletAutoFunds { name, role } => {
                    let wallet = defined_wallets
                        .iter()
                        .cloned()
                        .find(|(x, _)| {
                            *x.to_lowercase() == name.to_lowercase()
                        })
                        .map(|(_, y)| y.clone())
                        .ok_or_else(|| Error::CannotFindAlias(name.to_string()))?;
                    voter_hirs.push(VoterHIR {
                        voting_power: (*wallet.template().value()).into(),
                        voting_key: Wallet::from(wallet).account_id(),
                        voting_group: role.to_string(),
                    });
                }
                Initial::External { key, funds, role } => {
                    voter_hirs.push(VoterHIR {
                        voting_key: Identifier::from_hex(key)?,
                        voting_group: role.to_string(),
                        voting_power: (*funds).into(),
                    });
                }
            }
        }
        Ok(voter_hirs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    Random {
        count: usize,
        level: u64,
        #[serde(default)]
        role: Role,
    },
    WalletAutoFunds {
        name: String,
        #[serde(default)]
        role: Role,
    },
    Wallet {
        name: String,
        funds: u64,
        #[serde(default)]
        role: Role,
    },
    External {
        key: String,
        funds: u64,
        #[serde(default)]
        role: Role,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cannot find alias '{0}' in snapshot initials")]
    CannotFindAlias(String),
    #[error(transparent)]
    PublicKey(#[from] PublicKeyFromStrError),
}
