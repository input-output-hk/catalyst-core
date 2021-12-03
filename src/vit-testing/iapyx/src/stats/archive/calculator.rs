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
    pub fn calculate_distribution<T>(
        result: &BTreeMap<T, usize>,
    ) -> Result<BTreeMap<usize, usize>, ArchiveCalculatorError> {
        println!("calculate distribution");

        let mut distribution = vec![
            0usize;
            *result
                .values()
                .max()
                .ok_or(ArchiveCalculatorError::CannotCalculateMaxElement)?
                + 1
        ];

        for count in result.values() {
            distribution[*count] += 1;
        }

        Ok(distribution
            .iter()
            .enumerate()
            .map(|(idx, count)| (idx, *count))
            .collect())
    }

    pub fn number_of_tx_per_slot(&self) -> Result<BTreeMap<String, usize>, ArchiveCalculatorError> {
        println!("calculating number of tx per slot..");
        Ok(self
            .records
            .iter()
            .group_by(|item| item.time.to_string())
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect())
    }

    pub fn distinct_casters(&self) -> Result<BTreeSet<String>, ArchiveCalculatorError> {
        println!("calculating distinct caster..");
        Ok(self
            .records
            .iter()
            .group_by(|item| item.caster.to_string())
            .into_iter()
            .map(|(key, _group)| key)
            .collect())
    }

    pub fn number_of_votes_per_caster(
        &self,
    ) -> Result<BTreeMap<String, usize>, ArchiveCalculatorError> {
        println!("calculating number of votes per caster..");
        Ok(self
            .records
            .iter()
            .group_by(|item| item.caster.to_string())
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect())
    }

    pub fn batch_size_per_caster(
        &self,
        slots_in_epoch: u32,
    ) -> Result<BTreeMap<String, usize>, ArchiveCalculatorError> {
        println!("calculating batch size per caster..");

        let time_era = self.records[0].block_date()?.time_era(slots_in_epoch);

        Ok(self
            .records
            .iter()
            .group_by(|item| item.caster.to_string())
            .into_iter()
            .map(|(key, group)| {
                let mut sorted_group: Vec<&ArchiverRecord> = group.collect();

                sorted_group.sort_by_key(|a| a.block_date().unwrap());

                let mut max_batch_size = 1;
                let mut current_batch_size = 1;
                let mut last_slot: BlockDate = sorted_group[0].block_date().unwrap();

                for item in sorted_group.iter().skip(1) {
                    let current: BlockDate = item.block_date().unwrap();
                    if are_equal_or_adherent(&last_slot, &current, &time_era) {
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

fn are_equal_or_adherent(left: &BlockDate, right: &BlockDate, time_era: &TimeEra) -> bool {
    left == right || left.clone().shift_slot(1, time_era) == *right
}

#[derive(Debug, Error)]
pub enum ArchiveCalculatorError {
    #[error("general error")]
    General(#[from] std::io::Error),
    #[error("cannot calculate distribution: cannot calculate max element")]
    CannotCalculateMaxElement,
    #[error("csv error")]
    Csv(#[from] csv::Error),
    #[error("block date error")]
    BlockDateParse(#[from] BlockDateParseError),
}
