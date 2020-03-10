use crate::header::Epoch;
use crate::value::Value;
use chain_ser::deser::{Serialize, Deserialize};
use chain_ser::packer::Codec;
use std::io::Error;

/// Last rewards associated with a state
///
/// It tracks the epoch where the rewards has been received,
/// and the total amount of reward for such an epoch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastRewards {
    pub epoch: Epoch,
    pub reward: Value,
}

impl Serialize for LastRewards {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        let mut codec = Codec::new(writer);
        codec.put_u32(self.epoch)?;
        codec.put_u64(self.reward.0)?;
        Ok(())
    }
}

impl Deserialize for LastRewards {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let mut codec = Codec::new(reader);
        Ok(LastRewards {
            epoch: codec.get_u32()?,
            reward: Value(codec.get_u64()?)
        })
    }
}

impl LastRewards {
    /// Create an initial value of epoch=0 reward=0
    ///
    /// It is also safe as the "uninitialized" value, since
    /// epoch 0 doesn't have by construction any reward associated.
    pub fn default() -> Self {
        LastRewards {
            epoch: 0,
            reward: Value::zero(),
        }
    }

    /// Add some value to the last reward, if the epoch is the same, then the
    /// result is just added, however.account
    ///
    /// This should never be used with an epoch less than the last set epoch,
    /// as it would means the rewards system is rewarding something from a past state.
    pub fn add_for(&mut self, epoch: Epoch, value: Value) {
        assert!(epoch >= self.epoch);
        if self.epoch == epoch {
            self.reward = (self.reward + value).unwrap()
        } else {
            self.epoch = epoch;
            self.reward = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    pub fn add_for_new_epoch_replaced_old_value() {
        let value_to_add = Value(100);
        let mut last_rewards = LastRewards {
            epoch: 0,
            reward: Value(50),
        };
        last_rewards.add_for(1, value_to_add);
        assert_eq!(
            last_rewards.reward,
            Value(100),
            "incorrect value for rewards {} vs {}",
            last_rewards.reward,
            value_to_add
        );
    }

    #[test]
    pub fn add_for_current_epoch_increment_value() {
        let value_to_add = Value(100);
        let epoch = 1;
        let mut last_rewards = LastRewards {
            epoch: 1,
            reward: Value(50),
        };
        last_rewards.add_for(epoch, value_to_add);
        assert_eq!(
            last_rewards.reward,
            Value(150),
            "incorrect value for rewards {} vs {}",
            last_rewards.reward,
            value_to_add
        );
    }

    #[test]
    #[should_panic]
    pub fn add_for_wrong_epoch() {
        let value_to_add = Value(100);
        let epoch = 1;
        let mut last_rewards = LastRewards {
            epoch: 2,
            reward: Value::zero(),
        };
        last_rewards.add_for(epoch, value_to_add);
    }

    #[test]
    #[should_panic]
    pub fn add_for_value_overflow() {
        let value_to_add = Value(std::u64::MAX);
        let epoch = 0;
        let mut last_rewards = LastRewards {
            epoch: 0,
            reward: Value(std::u64::MAX),
        };
        last_rewards.add_for(epoch, value_to_add);
    }

    #[test]
    pub fn last_rewards_serialize_deserialize() -> Result<(), std::io::Error> {
        use std::io::Cursor;

        let last_rewards = LastRewards {
            epoch: 0,
            reward: Value(1),
        };

        let mut c: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        last_rewards.serialize(&mut c)?;
        c.set_position(0);
        let deserialize_last_rewards = LastRewards::deserialize(&mut c)?;
        assert_eq!(last_rewards, deserialize_last_rewards);
        Ok(())
    }
}
