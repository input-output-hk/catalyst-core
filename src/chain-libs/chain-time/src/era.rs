//! Split timeframe in eras

use crate::timeframe::Slot;
use chain_core::{
    packer::Codec,
    property::{ReadError, WriteError},
};
use std::fmt;

/// Epoch number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct Epoch(pub u32);

/// Slot Offset *in* a given epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct EpochSlotOffset(pub u32);

/// Epoch position: this is an epoch and a slot offset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct EpochPosition {
    pub epoch: Epoch,
    pub slot: EpochSlotOffset,
}

impl fmt::Display for EpochPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.epoch.0, self.slot.0)
    }
}

/// Describe a new era, which start at epoch_start and is associated
/// to a specific slot. Each epoch have a constant number of slots on a given time era.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct TimeEra {
    epoch_start: Epoch,
    slot_start: Slot,
    slots_per_epoch: u32,
}

pub fn pack_time_era<W: std::io::Write>(
    time_era: &TimeEra,
    codec: &mut Codec<W>,
) -> Result<(), WriteError> {
    codec.put_be_u32(time_era.epoch_start.0)?;
    codec.put_be_u64(time_era.slot_start.0)?;
    codec.put_be_u32(time_era.slots_per_epoch)?;
    Ok(())
}

pub fn unpack_time_era<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<TimeEra, ReadError> {
    let epoch_start = Epoch(codec.get_be_u32()?);
    let slot_start = Slot(codec.get_be_u64()?);
    let slots_per_epoch = codec.get_be_u32()?;

    Ok(TimeEra {
        epoch_start,
        slot_start,
        slots_per_epoch,
    })
}

impl TimeEra {
    /// Set a new era to start on slot_start at epoch_start for a given slots per epoch.
    pub fn new(slot_start: Slot, epoch_start: Epoch, slots_per_epoch: u32) -> Self {
        TimeEra {
            epoch_start,
            slot_start,
            slots_per_epoch,
        }
    }

    /// retrieve the epoch start during a given Era
    pub fn epoch_start(&self) -> Epoch {
        self.epoch_start
    }

    /// retrieve the slot start of an epoch during a given Era
    pub fn slot_start(&self) -> Slot {
        self.slot_start
    }

    /// retrieve the number of slots in an epoch during a given Era
    pub fn slots_per_epoch(&self) -> u32 {
        self.slots_per_epoch
    }

    /// Try to return the epoch/inner-epoch-slot associated.
    ///
    /// If the slot in parameter is before the beginning of this era, then
    /// None is returned.
    pub fn from_slot_to_era(&self, slot: Slot) -> Option<EpochPosition> {
        if slot < self.slot_start {
            return None;
        }
        let slot_era_offset = slot.0 - self.slot_start.0;
        let spe = self.slots_per_epoch as u64;
        let epoch_offset = (slot_era_offset / spe) as u32;
        let slot_offset = (slot_era_offset % spe) as u32;
        Some(EpochPosition {
            epoch: Epoch(self.epoch_start.0 + epoch_offset),
            slot: EpochSlotOffset(slot_offset),
        })
    }

    /// Convert an epoch position into a flat slot
    pub fn from_era_to_slot(&self, pos: EpochPosition) -> Slot {
        assert!(pos.epoch >= self.epoch_start);
        assert!(pos.slot.0 < self.slots_per_epoch);

        let slot_offset = (pos.epoch.0 as u64) * (self.slots_per_epoch as u64) + pos.slot.0 as u64;
        Slot(self.slot_start.0 + slot_offset)
    }
}

#[cfg(any(test))]
mod test {
    use super::*;
    use crate::timeframe::*;
    use crate::timeline::Timeline;

    use std::time::{Duration, SystemTime};

    use chain_ser::packer::Codec;
    use proptest::prelude::*;
    use test_strategy::proptest;

    #[proptest]
    fn time_era_pack_unpack_bijection(time_era: TimeEra) {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut codec = Codec::new(cursor);
        pack_time_era(&time_era, &mut codec).unwrap();
        let mut cursor = codec.into_inner();
        cursor.set_position(0);
        codec = Codec::new(cursor);
        let other_time_era = unpack_time_era(&mut codec).unwrap();
        prop_assert_eq!(time_era, other_time_era);
    }

    #[proptest]
    #[should_panic]
    //BUG_ID NPG-1002 owerflow look at fn time_era_from_era_to_slot_overflow()
    //BUG_ID NPG-1001 attempt to divide by zero when slots_per_epoch = 0
    fn time_era_slot_era_bijection(slot: Slot, era: TimeEra) {
        match era.from_slot_to_era(slot) {
            Some(epoch_pos) => {
                let other_slot = era.from_era_to_slot(epoch_pos);
                prop_assert_eq!(slot, other_slot);
            }
            None => {
                prop_assert!(slot < era.slot_start);
            }
        }
    }

    #[proptest]
    #[should_panic]
    //BUG_ID NPG-1003 Epoch is u32 but should be u64
    fn time_era_slot_to_era(slot: Slot) {
        let slot_start = Slot(0);
        let epoch_start = Epoch(0);
        let slots_per_epoch = 1;
        let era = TimeEra::new(slot_start, epoch_start, slots_per_epoch);
        let epoch_pos = era.from_slot_to_era(slot).unwrap();
        //with one slot per epoch the input slot will be equal to the epoch
        prop_assert_eq!(slot.0, epoch_pos.epoch.0.into());
    }

    #[proptest]
    #[should_panic]
    //BUG_ID NPG-1000 from_era_to_slot should handle the error returning <Result> instead of panicking
    fn time_era_from_era_to_slot(epoch_pos: EpochPosition) {
        let slot_start = Slot(0);
        let epoch_start = Epoch(0);
        let slots_per_epoch = 1;
        let era = TimeEra::new(slot_start, epoch_start, slots_per_epoch);
        let slot = era.from_era_to_slot(epoch_pos);
        //with one slot per epoch the input slot will be equal to the epoch
        prop_assert_eq!(slot.0, (epoch_pos.epoch.0 + epoch_pos.slot.0) as u64);
    }

    #[test]
    pub fn it_works() {
        let now = SystemTime::now();
        let t0 = Timeline::new(now);

        let f0 = SlotDuration::from_secs(5);

        let tf0 = TimeFrame::new(t0, f0);

        let t1 = now + Duration::from_secs(10);
        let t2 = now + Duration::from_secs(20);
        let t3 = now + Duration::from_secs(100);

        let slot1 = tf0.slot_at(&t1).unwrap();
        let slot2 = tf0.slot_at(&t2).unwrap();
        let slot3 = tf0.slot_at(&t3).unwrap();

        assert_eq!(slot1, Slot(2));
        assert_eq!(slot2, Slot(4));
        assert_eq!(slot3, Slot(20));

        let era = TimeEra::new(slot1, Epoch(2), 4);

        let p1 = era.from_slot_to_era(slot1).unwrap();
        let p2 = era.from_slot_to_era(slot2).unwrap();
        let p3 = era.from_slot_to_era(slot3).unwrap();

        assert_eq!(
            p1,
            EpochPosition {
                epoch: Epoch(2),
                slot: EpochSlotOffset(0)
            }
        );
        assert_eq!(
            p2,
            EpochPosition {
                epoch: Epoch(2),
                slot: EpochSlotOffset(2)
            }
        );
        // 20 - 2 => 18 / 4 => era_start(2) + (4, 2)
        assert_eq!(
            p3,
            EpochPosition {
                epoch: Epoch(6),
                slot: EpochSlotOffset(2)
            }
        );
    }
}
