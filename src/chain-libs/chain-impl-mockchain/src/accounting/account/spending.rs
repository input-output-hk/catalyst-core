//! Spending strategies
use super::LedgerError;

/// Simple strategy to spend from multiple increasing counters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingCounterIncreasing {
    nexts: Vec<SpendingCounter>,
}

// number of bits reserved for lanes
const LANES_BITS: usize = 3;
// number of bits reserved for counter (unrespective of the lane)
const UNLANES_BITS: usize = 32 - LANES_BITS;

impl SpendingCounterIncreasing {
    /// number of parallel lanes of increasing counters
    pub const LANES: usize = 1 << LANES_BITS;

    pub fn new_from_counter(set: SpendingCounter) -> Self {
        let mut x = Self::default();
        x.nexts[set.lane()] = set;
        x
    }

    pub fn new_from_counters(set: Vec<SpendingCounter>) -> Option<Self> {
        if set.len() == Self::LANES {
            for (i, i_set_value) in set.iter().enumerate() {
                if i_set_value.lane() != i {
                    return None;
                }
            }
            Some(SpendingCounterIncreasing { nexts: set })
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

    /// try to match the lane of the counter in argument, if it doesn't match
    /// a ledger error reported.
    ///
    /// If the counter match succesfully, then the counter at this lane is incremented by one.
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
///
/// Note that the leading LANES_BITS bits are used to codify the
/// implicit lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendingCounter(pub(crate) u32);

impl SpendingCounter {
    // on 32 bits: 0x1fff_ffff;
    const UNLANED_MASK: u32 = (1 << UNLANES_BITS) - 1;

    // LANES_BITS on the MSB, on 32 bits: 0xe000_0000
    const LANED_MASK: u32 = !Self::UNLANED_MASK;

    pub fn lane(self) -> usize {
        (self.0 >> UNLANES_BITS) as usize
    }

    pub fn unlaned_counter(self) -> u32 {
        self.0 & Self::UNLANED_MASK
    }

    pub fn new(lane: usize, counter: u32) -> Self {
        assert!(lane < (1 << LANES_BITS));
        assert!(counter < (1 << UNLANES_BITS));
        SpendingCounter((lane << UNLANES_BITS) as u32 | (counter & Self::UNLANED_MASK))
    }

    pub fn zero() -> Self {
        SpendingCounter(0)
    }

    /// Increment the counter within it own lane. the lane of where this counter apply, cannot change
    /// through the incrementation procedure
    ///
    /// if the counter bits overflow, it will automatically be wrapped, so
    /// that the lane remains identical
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;

    #[quickcheck_macros::quickcheck]
    fn spending_counter_serialization_bijection(sc: SpendingCounter) -> TestResult {
        let bytes = sc.to_bytes();
        TestResult::from_bool(SpendingCounter::from_bytes(bytes) == sc)
    }

    #[test]
    #[should_panic]
    fn new_invalid_spending_counter() {
        let lane: usize = (1 << LANES_BITS) + 1;
        let counter: u32 = 1 << UNLANES_BITS;
        SpendingCounter::new(lane, counter);
    }

    #[quickcheck_macros::quickcheck]
    fn new_spending_counter(mut lane: usize, mut counter: u32) {
        lane = lane % (1 << LANES_BITS);
        counter = counter % (1 << UNLANES_BITS);
        let sc = SpendingCounter::new(lane, counter);

        assert_eq!(lane, sc.lane());
        assert_eq!(counter, sc.unlaned_counter());
    }

    #[quickcheck_macros::quickcheck]
    fn increment_counter(mut spending_counter: SpendingCounter) -> TestResult {
        if spending_counter.unlaned_counter().checked_add(1).is_none() {
            return TestResult::discard();
        }
        let lane_before = spending_counter.lane();
        let counter = spending_counter.unlaned_counter();
        spending_counter = spending_counter.increment();
        assert_eq!(lane_before, spending_counter.lane());
        TestResult::from_bool((counter + 1) == spending_counter.unlaned_counter())
    }

    #[quickcheck_macros::quickcheck]
    pub fn increment_nth(mut spending_counter: SpendingCounter, n: u32) -> TestResult {
        if spending_counter.unlaned_counter().checked_add(n).is_none() {
            return TestResult::discard();
        }
        let lane_before = spending_counter.lane();
        let counter = spending_counter.unlaned_counter();
        spending_counter = spending_counter.increment_nth(n);
        assert_eq!(lane_before, spending_counter.lane());
        TestResult::from_bool((counter + n) == spending_counter.unlaned_counter())
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn increment_counter_overflow_debug() {
        let _ = SpendingCounter::new(8, u32::MAX).increment();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    pub fn increment_nth_overflow_debug() {
        let _ = SpendingCounter::new(0, 1).increment_nth(u32::MAX);
    }

    #[quickcheck_macros::quickcheck]
    pub fn spending_counter_is_set_on_correct_lane(
        spending_counter: SpendingCounter,
    ) -> TestResult {
        let spending_counter_increasing =
            SpendingCounterIncreasing::new_from_counter(spending_counter);
        let counters = spending_counter_increasing.get_valid_counters();
        TestResult::from_bool(spending_counter == counters[spending_counter.lane()])
    }

    #[test]
    pub fn spending_counters_duplication() {
        let counters = vec![SpendingCounter::zero(), SpendingCounter::zero()];
        assert!(SpendingCounterIncreasing::new_from_counters(counters).is_none());
    }

    #[test]
    pub fn spending_counters_incorrect_order() {
        let counters = vec![SpendingCounter::new(1, 0), SpendingCounter::new(0, 0)];
        assert!(SpendingCounterIncreasing::new_from_counters(counters).is_none());
    }

    #[test]
    pub fn spending_counters_too_many_sub_counters() {
        let counters = std::iter::from_fn(|| Some(SpendingCounter::zero()))
            .take(SpendingCounterIncreasing::LANES + 1)
            .collect();
        assert!(SpendingCounterIncreasing::new_from_counters(counters).is_none());
    }

    #[quickcheck_macros::quickcheck]
    pub fn spending_counter_increasing_increment(mut index: usize) -> TestResult {
        let mut sc_increasing = SpendingCounterIncreasing::default();
        index %= SpendingCounterIncreasing::LANES;
        let sc_before = sc_increasing.get_valid_counters()[index];
        sc_increasing.next_verify(sc_before).unwrap();

        let sc_after = sc_increasing.get_valid_counters()[index];
        TestResult::from_bool(sc_after.unlaned_counter() == sc_before.unlaned_counter() + 1)
    }

    #[test]
    pub fn spending_counter_increasing_wrong_counter() {
        let mut sc_increasing = SpendingCounterIncreasing::default();
        let incorrect_sc = SpendingCounter::new(0, 100);
        assert!(sc_increasing.next_verify(incorrect_sc).is_err());
    }

    #[test]
    #[should_panic]
    pub fn spending_counter_increasing_wrong_lane() {
        let mut sc_increasing = SpendingCounterIncreasing::default();
        let incorrect_sc = SpendingCounter::new(SpendingCounterIncreasing::LANES, 1);
        assert!(sc_increasing.next_verify(incorrect_sc).is_err());
    }
}
