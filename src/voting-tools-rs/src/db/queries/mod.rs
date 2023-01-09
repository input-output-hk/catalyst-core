use crate::data_provider::DataProvider;
use crate::data::{SlotNo, Reg};
use crate::Db;
use bigdecimal::BigDecimal;
use std::collections::HashMap;

mod stake_value;
mod vote_registrations;

impl DataProvider for Db {
    fn vote_registrations(
        &self,
        lower: Option<SlotNo>,
        upper: Option<SlotNo>,
    ) -> color_eyre::Result<Vec<Reg>> {
        self.vote_registrations(lower, upper)
    }

    fn stake_values<'a>(
        &self,
        stake_addrs: &'a [String],
    ) -> color_eyre::Result<HashMap<&'a str, BigDecimal>> {
        self.stake_values(stake_addrs)
    }
}
