//! Spending strategies
use super::LedgerError;

/// Simple strategy to spend from an increasing counter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingCounterIncreasing {
    nexts: Vec<SpendingCounter>,
}

impl SpendingCounterIncreasing {
    /// number of parallel lanes of increasing counters
    pub const LANES: usize = 8;

    pub fn new_from_counter(set: SpendingCounter) -> Self {
        let mut x = Self::default();
        x.nexts[set.lane()] = set;
        x
    }

    pub fn new_from_counters(set: Vec<SpendingCounter>) -> Option<Self> {
        if set.len() == Self::LANES {
            for i in 0..set.len() {
                if set[i].lane() != i {
                    return None;
                }
            }
            Some(SpendingCounterIncreasing { nexts: set.into() })
        } else {
            None
        }
    }

    pub fn get_valid_counter(&self) -> SpendingCounter {
        self.nexts[0]
    }

    pub fn get_valid_counters(&self) -> Vec<SpendingCounter> {
        self.nexts.clone()
    }

    pub fn next_verify(&mut self, counter: SpendingCounter) -> Result<(), LedgerError> {
        let actual_counter = self.nexts[counter.lane()];

        if actual_counter != counter {
            Err(LedgerError::SpendingCredentialInvalid)
        } else {
            self.nexts[counter.lane()] = actual_counter.increment();
            Ok(())
        }
    }
}

// only used to print the account's ledger
impl std::fmt::Display for SpendingCounterIncreasing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.nexts.iter() {
            write!(f, "{},", c.0)?
        }
        Ok(())
    }
}

impl Default for SpendingCounterIncreasing {
    fn default() -> Self {
        let mut nexts = Vec::new();
        for i in 0..Self::LANES {
            nexts.push(SpendingCounter::new(i, 0));
        }
        SpendingCounterIncreasing { nexts }
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
    const UNLANED_MASK: u32 = 0x1fff_ffff;
    const LANED_MASK: u32 = 0xe000_0000;

    fn lane(self) -> usize {
        (self.0 >> 29) as usize
    }

    fn unlaned_counter(self) -> u32 {
        self.0 & Self::UNLANED_MASK
    }

    pub(crate) fn new(lane: usize, counter: u32) -> Self {
        assert!(lane < 0b1000);
        assert!(counter < 0x2000_0000);
        SpendingCounter((lane << 29) as u32 | (counter & Self::UNLANED_MASK))
    }

    pub fn zero() -> Self {
        SpendingCounter(0)
    }

    #[must_use = "this function does not modify the state"]
    pub fn increment(self) -> Self {
        let inc = (self.unlaned_counter() + 1) & Self::UNLANED_MASK;
        SpendingCounter((self.0 & Self::LANED_MASK) | inc)
    }

    #[must_use = "this function does not modify the state"]
    pub fn increment_nth(self, n: u32) -> Self {
        let inc = (self.unlaned_counter() + n) & Self::UNLANED_MASK;
        SpendingCounter((self.0 & Self::LANED_MASK) | inc)
    }

    pub fn to_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(bytes))
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
