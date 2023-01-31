use crate::data::{SignedRegistration, SlotNo, StakeKeyHex};
use crate::data_provider::DataProvider;
use crate::Db;
use bigdecimal::BigDecimal;
use std::collections::HashMap;

mod stake_value;
mod vote_registrations;

impl DataProvider for Db {
    fn vote_registrations(
        &self,
        lower: SlotNo,
        upper: SlotNo,
    ) -> color_eyre::Result<Vec<SignedRegistration>> {
        self.vote_registrations(lower, upper)
    }

    fn stake_values(
        &self,
        stake_addrs: &[StakeKeyHex],
    ) -> color_eyre::Result<HashMap<StakeKeyHex, BigDecimal>> {
        self.stake_values(stake_addrs)
    }
}
