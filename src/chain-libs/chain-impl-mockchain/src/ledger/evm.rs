#[cfg(feature = "evm")]
use chain_evm::state::{AccountTrie, Balance};

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Ledger {
    #[cfg(feature = "evm")]
    pub(crate) accounts: AccountTrie,
}

impl Ledger {
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(not(feature = "evm"))]
impl Ledger {
    pub(crate) fn stats(&self) -> Option<String> {
        None
    }

    pub(crate) fn info_eq(&self, _other: &Self) -> Option<String> {
        None
    }
}

#[cfg(feature = "evm")]
impl Ledger {
    pub(crate) fn stats(&self) -> Option<String> {
        let Ledger { accounts } = self;
        let mut count = 0;
        let mut total = Balance::zero();
        for (_, account) in accounts {
            count += 1;
            total += account.balance;
        }
        Some(format!("EVM accounts: #{} Total={:?}", count, total))
    }

    pub(crate) fn info_eq(&self, other: &Self) -> Option<String> {
        Some(format!("evm: {}", self.accounts == other.accounts))
    }
}
