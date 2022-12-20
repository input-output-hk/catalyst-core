//! Chain-time
//!
//! Utilities for working with time
#![warn(clippy::all)]
#![deny(missing_docs)]

pub mod era;
pub mod timeframe;
pub mod timeline;
pub mod units;

pub use era::{Epoch, EpochPosition, EpochSlotOffset, TimeEra};
pub use timeframe::{Slot, SlotDuration, TimeFrame};
pub use timeline::{TimeOffsetSeconds, Timeline};
pub use units::DurationSeconds;

#[cfg(any(test, feature = "property-test-api"))]
#[doc(hidden)]
pub mod testing;
