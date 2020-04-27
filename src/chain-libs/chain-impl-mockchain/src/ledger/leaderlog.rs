use crate::certificate::PoolId;
use imhamt::{Hamt, HamtIter, InsertError};
use std::collections::hash_map::DefaultHasher;

/// Count how many blocks have been created by a specific Pool
#[derive(Clone, PartialEq, Eq)]
pub struct LeadersParticipationRecord {
    total: u32,
    log: Hamt<DefaultHasher, PoolId, u32>,
}

impl Default for LeadersParticipationRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl LeadersParticipationRecord {
    pub fn total(&self) -> u32 {
        self.total
    }

    pub fn nb_participants(&self) -> usize {
        self.log.size()
    }

    /// new empty leader log
    pub fn new() -> Self {
        Self {
            total: 0,
            log: Hamt::new(),
        }
    }

    /// Add one count to a pool. if the pool doesn't exist, then set it to 1
    pub fn increase_for(&mut self, pool: &PoolId) {
        self.total += 1;
        self.log = self
            .log
            .insert_or_update_simple(pool.clone(), 1, |v| Some(v + 1));
    }

    /// Set a pool id to a specific value.
    ///
    /// if the value already exists, then it returns an insert error.
    /// This should only be used related to the iterator construction,
    pub fn set_for(&mut self, pool: PoolId, v: u32) -> Result<(), InsertError> {
        self.log = self.log.insert(pool, v)?;
        self.total += v;
        Ok(())
    }

    /// Iterate over all known pool record
    pub fn iter(&self) -> HamtIter<'_, PoolId, u32> {
        self.log.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{certificate::PoolId, testing::builders::StakePoolBuilder};

    #[test]
    pub fn test_total() {
        let leaders_participation_record = create_log(vec![
            (new_stake_pool_id(), 10),
            (new_stake_pool_id(), 0),
            (new_stake_pool_id(), 11),
            (new_stake_pool_id(), 5),
        ]);
        verify_participants_count(&leaders_participation_record, 4);
        verify_total(&leaders_participation_record, 26);
    }

    #[test]
    pub fn test_increase_for_existing_pool() {
        let stake_pool_id = new_stake_pool_id();

        let mut leaders_participation_record = create_log(vec![(stake_pool_id.clone(), 1)]);

        leaders_participation_record.increase_for(&stake_pool_id);

        assert_eq!(
            2,
            get_count(&leaders_participation_record, &stake_pool_id).unwrap(),
            "wrong count for existing stake pool"
        );
        verify_participants_count(&leaders_participation_record, 1);
        verify_total(&leaders_participation_record, 2);
    }

    #[test]
    pub fn test_increase_for_non_existing_pool() {
        let stake_pool_id = new_stake_pool_id();
        let non_existing_stake_pool_id = new_stake_pool_id();

        let mut leaders_participation_record = create_log(vec![(stake_pool_id.clone(), 1)]);

        leaders_participation_record.increase_for(&non_existing_stake_pool_id);

        assert_eq!(
            1,
            get_count(&leaders_participation_record, &non_existing_stake_pool_id).unwrap(),
            "wrong count for new stake pool"
        );
        assert_eq!(
            1,
            get_count(&leaders_participation_record, &stake_pool_id).unwrap(),
            "wrong count for existing stake pool"
        );

        verify_participants_count(&leaders_participation_record, 2);
        verify_total(&leaders_participation_record, 2);
    }

    #[test]
    pub fn test_set_new_value_for_existing_stake_pool() {
        let stake_pool_id = new_stake_pool_id();

        let mut leaders_participation_record = create_log(vec![(stake_pool_id.clone(), 1)]);

        assert_eq!(
            InsertError::EntryExists,
            leaders_participation_record
                .set_for(stake_pool_id.clone(), 10)
                .err()
                .unwrap()
        );

        assert_eq!(
            1,
            get_count(&leaders_participation_record, &stake_pool_id).unwrap(),
            "wrong count for new stake pool"
        );

        verify_participants_count(&leaders_participation_record, 1);
        verify_total(&leaders_participation_record, 1);
    }

    #[test]
    pub fn test_set_duplicated_value_for_existing_stake_pool() {
        let first_stake_pool_id = new_stake_pool_id();

        let mut leaders_participation_record = create_log(vec![(first_stake_pool_id.clone(), 1)]);

        assert_eq!(
            InsertError::EntryExists,
            leaders_participation_record
                .set_for(first_stake_pool_id, 1)
                .err()
                .unwrap()
        );
        verify_total(&leaders_participation_record, 1);
    }

    #[test]
    pub fn test_set_new_value_for_non_existing_stake_pool() {
        let stake_pool_id = new_stake_pool_id();

        let mut leaders_participation_record = create_log(vec![(new_stake_pool_id(), 1)]);

        leaders_participation_record
            .set_for(stake_pool_id.clone(), 10)
            .expect("unexpected panic when set for first stake pool");

        assert_eq!(
            10,
            get_count(&leaders_participation_record, &stake_pool_id).unwrap(),
            "wrong count for new stake pool"
        );

        verify_participants_count(&leaders_participation_record, 2);
        verify_total(&leaders_participation_record, 11);
    }

    fn create_log(records: Vec<(PoolId, u32)>) -> LeadersParticipationRecord {
        let mut leaders_participation_record = LeadersParticipationRecord::new();
        for (pool_id, count) in records.iter() {
            leaders_participation_record
                .set_for(pool_id.clone(), *count)
                .expect("panic when filling records");
        }
        leaders_participation_record
    }

    fn get_count(
        leaders_participation_record: &LeadersParticipationRecord,
        stake_pool_id: &PoolId,
    ) -> Option<u32> {
        leaders_participation_record
            .iter()
            .find(|x| *x.0 == *stake_pool_id)
            .map(|x| *x.1)
    }

    fn new_stake_pool_id() -> PoolId {
        StakePoolBuilder::new().build().id()
    }

    fn verify_total(leaders_participation_record: &LeadersParticipationRecord, count: u32) {
        let actual = leaders_participation_record.total();
        assert_eq!(actual, count, "wrong total value {} vs {}", actual, count);
    }

    fn verify_participants_count(
        leaders_participation_record: &LeadersParticipationRecord,
        count: u32,
    ) {
        let actual = leaders_participation_record.nb_participants();
        assert_eq!(
            actual, count as usize,
            "wrong no of participants {} vs {}",
            actual, count
        );
    }
}
