use chain_time::{Epoch, Slot, TimeEra};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(remote = "Epoch")]
pub struct EpochDef(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(remote = "Slot")]
pub struct SlotDef(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "TimeEra")]
pub struct TimeEraDef {
    #[serde(with = "EpochDef", getter = "TimeEra::epoch_start")]
    epoch_start: Epoch,
    #[serde(with = "SlotDef", getter = "TimeEra::slot_start")]
    slot_start: Slot,
    #[serde(getter = "TimeEra::slots_per_epoch")]
    slots_per_epoch: u32,
}

impl From<TimeEraDef> for TimeEra {
    fn from(val: TimeEraDef) -> Self {
        Self::new(val.slot_start, val.epoch_start, val.slots_per_epoch)
    }
}
