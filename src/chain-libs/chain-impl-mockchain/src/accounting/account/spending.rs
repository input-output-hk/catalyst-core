//! Spending strategies

/// Simple strategy to spend from an increasing counter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingCounterIncreasing {
    next: SpendingCounter,
}

impl SpendingCounterIncreasing {
    pub fn new_from_counter(set: SpendingCounter) -> Self {
        SpendingCounterIncreasing { next: set }
    }

    pub fn get_current_counter(&self) -> SpendingCounter {
        self.next
    }

    #[must_use = "this function does not modify the state"]
    pub fn next(&self) -> Self {
        SpendingCounterIncreasing {
            next: self.next.increment(),
        }
    }
}

// only used to print the account's ledger
impl std::fmt::Display for SpendingCounterIncreasing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.next.0)
    }
}

impl Default for SpendingCounterIncreasing {
    fn default() -> Self {
        SpendingCounterIncreasing {
            next: SpendingCounter::zero(),
        }
    }
}

/// Spending counter associated to an account.
///
/// every time the owner is spending from an account,
/// the counter is incremented. A matching counter
/// needs to be used in the spending phase to make
/// sure we have non-replayability of a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendingCounter(pub(crate) u32);

impl SpendingCounter {
    pub fn zero() -> Self {
        SpendingCounter(0)
    }

    #[must_use = "this function does not modify the state"]
    pub fn increment(self) -> Self {
        SpendingCounter(self.0.wrapping_add(1))
    }

    #[must_use = "this function does not modify the state"]
    pub fn increment_nth(self, n: u32) -> Self {
        SpendingCounter(self.0.wrapping_add(n))
    }

    pub fn to_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl From<u32> for SpendingCounter {
    fn from(v: u32) -> Self {
        SpendingCounter(v)
    }
}

impl From<SpendingCounter> for u32 {
    fn from(v: SpendingCounter) -> u32 {
        v.0
    }
}
