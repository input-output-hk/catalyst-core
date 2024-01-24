//! Spending strategies
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Spending credential invalid, expected {} got {} in lane {}", .expected.unlaned_counter(), .actual.unlaned_counter(), .actual.lane())]
    SpendingCredentialInvalid {
        expected: SpendingCounter,
        actual: SpendingCounter,
    },
    #[error("Invalid lane value during the SpendingCountersIncreasingInitialization, expected: {0}, got: {1}")]
    InvalidLaneValue(usize, usize),
    #[error("Invalid lane: {0} or counter: {1}, expected lane < (1 << LANES_BITS), counter < (1 << UNLANES_BITS)")]
    InvalidLaneOrCounter(usize, u32),
}

/// Simple strategy to spend from multiple increasing counters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendingCounterIncreasing {
    nexts: [SpendingCounter; Self::LANES],
}

// SpendingCounterIncreasing has extra invariants (e.g. nexts has 8 elements, each belongs to a different lane), so a derived implementation is not suitable

// number of bits reserved for lanes
const LANES_BITS: usize = 3;
// number of bits reserved for counter (unrespective of the lane)
const UNLANES_BITS: usize = 32 - LANES_BITS;

impl SpendingCounterIncreasing {
    /// number of parallel lanes of increasing counters, equals to 8
    pub const LANES: usize = 1 << LANES_BITS;

    pub fn new_from_counter(set: SpendingCounter) -> Self {
        let mut x = Self::default();
        x.nexts[set.lane()] = set;
        x
    }

    pub fn new_from_counters(
        nexts: [SpendingCounter; SpendingCounterIncreasing::LANES],
    ) -> Result<Self, Error> {
        for (i, i_set_value) in nexts.iter().enumerate() {
            if i_set_value.lane() != i {
                return Err(Error::InvalidLaneValue(i, i_set_value.lane()));
            }
        }
        Ok(SpendingCounterIncreasing { nexts })
    }

    pub fn get_valid_counter(&self) -> SpendingCounter {
        self.nexts[0]
    }

    pub fn get_valid_counters(&self) -> [SpendingCounter; Self::LANES] {
        self.nexts
    }

    /// try to match the lane of the counter in argument, if it doesn't match
    /// an error reported.
    ///
    /// If the counter match succesfully, then the counter at this lane is incremented by one.
    pub fn next_verify(&mut self, _counter: SpendingCounter) -> Result<(), Error> {
        // spending counter logic has been removed throughout, returning OK is the least invasive action at the moment.
        // Prod chain-libs = https://github.com/input-output-hk/chain-libs/tree/catalyst-fund9-gold
        Ok(())
    }

    /// Increases the spending counter on the given lane.
    pub(crate) fn next_unchecked(&mut self, _unchecked_counter: SpendingCounter) {}
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
        let nexts = [
            SpendingCounter::new(0, 0).unwrap(),
            SpendingCounter::new(1, 0).unwrap(),
            SpendingCounter::new(2, 0).unwrap(),
            SpendingCounter::new(3, 0).unwrap(),
            SpendingCounter::new(4, 0).unwrap(),
            SpendingCounter::new(5, 0).unwrap(),
            SpendingCounter::new(6, 0).unwrap(),
            SpendingCounter::new(7, 0).unwrap(),
        ];
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
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct SpendingCounter(u32);

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

    pub fn new(lane: usize, counter: u32) -> Result<Self, Error> {
        if lane < (1 << LANES_BITS) || counter < (1 << UNLANES_BITS) {
            Ok(SpendingCounter(
                (lane << UNLANES_BITS) as u32 | (counter & Self::UNLANED_MASK),
            ))
        } else {
            Err(Error::InvalidLaneOrCounter(lane, counter))
        }
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
    fn new_invalid_spending_counter() {
        let lane: usize = (1 << LANES_BITS) + 1;
        let counter: u32 = 1 << UNLANES_BITS;
        assert!(SpendingCounter::new(lane, counter).is_err());
    }

    #[quickcheck_macros::quickcheck]
    fn new_spending_counter(mut lane: usize, mut counter: u32) {
        lane %= 1 << LANES_BITS;
        counter %= 1 << UNLANES_BITS;
        let sc = SpendingCounter::new(lane, counter).unwrap();

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
        let _ = SpendingCounter::new(8, u32::MAX).unwrap().increment();
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    pub fn increment_nth_overflow_debug() {
        let _ = SpendingCounter::new(0, 1).unwrap().increment_nth(u32::MAX);
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
        let counters = [
            SpendingCounter::zero(),
            SpendingCounter::zero(),
            SpendingCounter::new(2, 0).unwrap(),
            SpendingCounter::new(3, 0).unwrap(),
            SpendingCounter::new(4, 0).unwrap(),
            SpendingCounter::new(5, 0).unwrap(),
            SpendingCounter::new(6, 0).unwrap(),
            SpendingCounter::new(7, 0).unwrap(),
        ];
        assert!(SpendingCounterIncreasing::new_from_counters(counters).is_err());
    }

    #[test]
    pub fn spending_counters_incorrect_order() {
        let counters = [
            SpendingCounter::new(1, 0).unwrap(),
            SpendingCounter::new(0, 0).unwrap(),
            SpendingCounter::new(2, 0).unwrap(),
            SpendingCounter::new(3, 0).unwrap(),
            SpendingCounter::new(4, 0).unwrap(),
            SpendingCounter::new(5, 0).unwrap(),
            SpendingCounter::new(6, 0).unwrap(),
            SpendingCounter::new(7, 0).unwrap(),
        ];
        assert!(SpendingCounterIncreasing::new_from_counters(counters).is_err());
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
        let incorrect_sc = SpendingCounter::new(0, 100).unwrap();
        assert!(sc_increasing.next_verify(incorrect_sc).is_err());
    }

    #[test]
    pub fn spending_counter_increasing_wrong_lane() {
        let mut sc_increasing = SpendingCounterIncreasing::default();
        let incorrect_sc = SpendingCounter::new(SpendingCounterIncreasing::LANES, 1).unwrap();
        assert!(sc_increasing.next_verify(incorrect_sc).is_err());
    }

    #[cfg(any(test, feature = "property-test-api"))]
    mod prop_impls {
        use proptest::prelude::*;

        use crate::{account::SpendingCounter, accounting::account::SpendingCounterIncreasing};

        impl Arbitrary for SpendingCounterIncreasing {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
                prop_oneof![
                    any::<SpendingCounter>()
                        .prop_map(|counter| Some(Self::new_from_counter(counter)))
                        .boxed(),
                    any::<[SpendingCounter; SpendingCounterIncreasing::LANES]>()
                        .prop_map(|counters| Self::new_from_counters(counters).ok())
                        .boxed(),
                ]
                .prop_filter_map("must be valid spending counter set", |i| i)
                .boxed()
            }
        }
    }
}
