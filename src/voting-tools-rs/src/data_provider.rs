use crate::model::{Reg, SlotNo};
use bigdecimal::BigDecimal;
use color_eyre::eyre::Result;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait DataProvider: Debug {
    fn vote_registrations(&self, lower: Option<SlotNo>, upper: Option<SlotNo>) -> Result<Vec<Reg>>;

    fn stake_values<'a>(&self, stake_addrs: &'a [String]) -> Result<HashMap<&'a str, BigDecimal>>;
}
