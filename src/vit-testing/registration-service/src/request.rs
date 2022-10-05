use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Request {
    pub payment_skey: String,
    pub payment_vkey: String,
    pub stake_skey: String,
    pub stake_vkey: String,
    pub legacy_skey: Option<String>,
    pub delegation_1: Option<String>,
    pub delegation_2: Option<String>,
    pub delegation_3: Option<String>,
}

impl Request {
    pub fn is_legacy(&self) -> bool {
        vec![&self.delegation_1, &self.delegation_2, &self.delegation_3]
            .iter()
            .all(|d| d.is_none())
    }

    pub fn delegations(&self) -> HashMap<String, u32> {
        vec![&self.delegation_1, &self.delegation_2, &self.delegation_3]
            .iter()
            .filter_map(|delegation| {
                if let Some(delegation) = delegation {
                    let mut tokens = delegation.split(',');
                    Some((
                        tokens.next().unwrap().to_string(),
                        tokens.next().unwrap().parse().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}
