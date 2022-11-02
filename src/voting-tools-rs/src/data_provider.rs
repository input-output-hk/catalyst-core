use crate::model::{Reg, SlotNo};
use bigdecimal::BigDecimal;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::fmt::Debug;

/// Abstraction trait over data provider for voting tools. This approach can allow various data sources
/// for registration queries like standard db sync database or in memory one including mocks.
pub trait DataProvider: Debug {
    /// Retrieves voter registration for optional bounds. They need to be expressed as absolute
    /// slot numbers
    ///
    /// # Errors
    ///
    /// Returns error on reading data issue
    fn vote_registrations(&self, lower: Option<SlotNo>, upper: Option<SlotNo>) -> Result<Vec<Reg>>;

    /// Retrieves stakes values for given array of addresses
    ///
    /// # Errors
    ///
    /// Returns error on reading data issue
    fn stake_values<'a>(&self, stake_addrs: &'a [String]) -> Result<HashMap<&'a str, BigDecimal>>;
}
