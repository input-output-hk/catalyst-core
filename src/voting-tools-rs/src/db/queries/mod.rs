use crate::data::{SignedRegistration, SlotNo};
use crate::data_provider::DataProvider;
use crate::verify::StakeKeyHash;
use crate::Db;
use bigdecimal::BigDecimal;
use dashmap::DashMap;

mod stake_value;
mod vote_registrations;
pub(crate) mod staked_utxo_ada;

impl DataProvider for Db {
    fn vote_registrations(
        &self,
        lower: SlotNo,
        upper: SlotNo,
    ) -> color_eyre::Result<Vec<SignedRegistration>> {
        self.vote_registrations(lower, upper)
    }

    fn stake_values(&self, stake_addrs: &[StakeKeyHash]) -> DashMap<StakeKeyHash, BigDecimal> {
        self.stake_values(stake_addrs)
    }
}
