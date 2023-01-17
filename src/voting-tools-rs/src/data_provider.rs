use crate::data::{Reg, SignedRegistration, SlotNo, StakeKeyHex};
use bigdecimal::BigDecimal;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::fmt::Debug;

/// Abstraction trait over data provider for voting tools. This approach can allow various data sources
/// for registration queries like standard db sync database or in memory one including mocks.
pub trait DataProvider: Debug {
    /// Get all vote registrations and signatures between two slot numbers
    ///
    /// If either slot number is `None`, they are ignored
    fn vote_registrations(
        &self,
        lower: Option<SlotNo>,
        upper: Option<SlotNo>,
    ) -> Result<Vec<SignedRegistration>>;

    /// Retrieves stakes values for given array of addresses
    fn stake_values(&self, stake_addrs: &[StakeKeyHex])
        -> Result<HashMap<StakeKeyHex, BigDecimal>>;
}

// Since we only need &self for all methods, we can implement DataProvider for any shared reference
// to a data provider
impl<T> DataProvider for &T
where
    T: DataProvider,
{
    fn vote_registrations(
        &self,
        lower: Option<SlotNo>,
        upper: Option<SlotNo>,
    ) -> Result<Vec<SignedRegistration>> {
        T::vote_registrations(self, lower, upper)
    }

    fn stake_values(
        &self,
        stake_addrs: &[StakeKeyHex],
    ) -> Result<HashMap<StakeKeyHex, BigDecimal>> {
        T::stake_values(self, stake_addrs)
    }
}
