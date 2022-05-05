use crate::config::initials::Role;
use chain_crypto::PublicKeyFromStrError;
use chain_impl_mockchain::value::Value;
use hersir::builder::Wallet as WalletSettings;
use jormungandr_lib::crypto::account::Identifier;
use serde::{Deserialize, Serialize};
use thor::{Wallet, WalletAlias};
use voting_hir::VoterHIR;

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
    pub fn as_voters_hirs(
        &self,
        defined_wallets: Vec<(&WalletAlias, &WalletSettings)>,
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
                            *x.to_lowercase() == format!("wallet_{}", name).to_lowercase()
                        })
                        .map(|(_, y)| y.clone())
                        .ok_or_else(|| Error::CannotFindAlias(name.to_string()))?;
                    voter_hirs.push(VoterHIR {
                        voting_power: funds
                            .map(Value)
                            .unwrap_or_else(|| *wallet.template().value())
                            .into(),
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
    Wallet {
        name: String,
        funds: Option<u64>,
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
