use super::loader::ArchiverRecord;
use chain_impl_mockchain::block::BlockDateParseError;
use chain_time::TimeEra;
use itertools::Itertools;
use jormungandr_lib::interfaces::BlockDate;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use thiserror::Error;

pub struct ArchiveStats {
    records: Vec<ArchiverRecord>,
}

impl From<Vec<ArchiverRecord>> for ArchiveStats {
    fn from(records: Vec<ArchiverRecord>) -> Self {
        Self { records }
    }
}

impl ArchiveStats {
    pub fn calculate_distribution(result: &BTreeMap<String, usize>) -> BTreeMap<usize, usize> {
        let mut distribution: BTreeMap<usize, usize> = BTreeMap::new();

        for value in result.values() {
            *distribution.entry(*value).or_default() += 1;
        }

        distribution
    }

    pub fn number_of_tx_per_slot(&self) -> BTreeMap<String, usize> {
        self.records
            .iter()
            .group_by(|item| item.time.to_string())
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect()
    }

    pub fn distinct_casters(&self) -> BTreeSet<String> {
        self.records
            .iter()
            .map(|item| item.caster.to_string())
            .collect()
    }

    pub fn number_of_votes_per_caster(&self) -> BTreeMap<String, usize> {
        self.records
            .iter()
            .group_by(|item| item.caster.to_string())
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect()
    }

    /// Method return max batch size calculated as biggest consecutive chain of transactions
    /// sent to blockchain.
    /// WARNING: single transaction in each slot would be counted as a batch also.
    pub fn max_batch_size_per_caster(
        &self,
        slots_in_epoch: u32,
    ) -> Result<BTreeMap<String, usize>, ArchiveCalculatorError> {
        let time_era = self.records[0].time.time_era(slots_in_epoch);

        Ok(self
            .records
            .iter()
            .group_by(|item| item.caster.to_string())
            .into_iter()
            .map(|(key, group)| {
                let mut sorted_group: Vec<&ArchiverRecord> = group.collect();

                sorted_group.sort_by_key(|a| a.time);

                let mut max_batch_size = 1;
                let mut current_batch_size = 1;
                let mut last_slot: BlockDate = sorted_group[0].time;

                for item in sorted_group.iter().skip(1) {
                    let current: BlockDate = item.time;
                    if are_equal_or_adjacent(&last_slot, &current, &time_era) {
                        current_batch_size += 1;
                    } else {
                        max_batch_size = std::cmp::max(max_batch_size, current_batch_size);
                        current_batch_size = 1;
                    }
                    last_slot = current;
                }
                (key, std::cmp::max(max_batch_size, current_batch_size))
            })
            .collect())
    }
}

fn are_equal_or_adjacent(left: &BlockDate, right: &BlockDate, time_era: &TimeEra) -> bool {
    left == right || left.clone().shift_slot(1, time_era) == *right
}

#[derive(Debug, Error)]
pub enum ArchiveCalculatorError {
    #[error("general error")]
    General(#[from] std::io::Error),
    #[error("cannot calculate distribution: cannot calculate max element result is empty")]
    EmptyResult,
    #[error("csv error")]
    Csv(#[from] csv::Error),
    #[error("block date error")]
    BlockDateParse(#[from] BlockDateParseError),
}
