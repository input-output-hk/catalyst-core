//! Split timeframe in eras

use crate::timeframe::Slot;
use chain_ser::packer::Codec;
use std::fmt;

/// Epoch number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u32);

/// Slot Offset *in* a given epoch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpochSlotOffset(pub u32);

/// Epoch position: this is an epoch and a slot offset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
pub struct TimeEra {
    epoch_start: Epoch,
    slot_start: Slot,
    slots_per_epoch: u32,
}

pub fn pack_time_era<W: std::io::Write>(
    time_era: &TimeEra,
    codec: &mut Codec<W>,
) -> Result<(), std::io::Error> {
    codec.put_u32(time_era.epoch_start.0)?;
    codec.put_u64(time_era.slot_start.0)?;
    codec.put_u32(time_era.slots_per_epoch)?;
    Ok(())
}

pub fn unpack_time_era<R: std::io::BufRead>(
    codec: &mut Codec<R>,
) -> Result<TimeEra, std::io::Error> {
    let epoch_start = Epoch(codec.get_u32()?);
    let slot_start = Slot(codec.get_u64()?);
    let slots_per_epoch = codec.get_u32()?;

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

    /// retrieve the number of slots in an epoch during a given Epoch
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

    use chain_ser::packer::Codec;
    use quickcheck::{quickcheck, TestResult};
    use std::io::Cursor;
    use std::time::{Duration, SystemTime};

    quickcheck! {
        fn time_era_pack_unpack_bijection(time_era: TimeEra) -> TestResult {
            let mut c : Cursor<Vec<u8>> = Cursor::new(Vec::new());
            let mut codec = Codec::new(c);
            match pack_time_era(&time_era, &mut codec) {
                Ok(_) => (),
                Err(e) => return TestResult::error(format!("{}", e)),
            }
            c = codec.into_inner();
            c.set_position(0);
            codec = Codec::new(c);
            match unpack_time_era(&mut codec) {
                Ok(other_time_era) => {
                    TestResult::from_bool(time_era == other_time_era)
                },
                Err(e) => TestResult::error(format!("{}", e)),
            }
        }
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
