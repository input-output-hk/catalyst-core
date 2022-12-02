use chain_impl_mockchain::config::Block0Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Block0Date")]
pub struct Block0DateDef(pub u64);
