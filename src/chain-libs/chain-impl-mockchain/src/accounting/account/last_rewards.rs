use crate::header::Epoch;
use crate::value::Value;
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


fn pack_last_rewards<W: std::io::Write>(last_rewards: &LastRewards, codec: &mut Codec<W>) -> Result<(), std::io::Error> {
    codec.put_u32(last_rewards.epoch)?;
    codec.put_u64(last_rewards.reward.0)?;
    Ok(())
}

fn unpack_last_rewards<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<LastRewards, std::io::Error> {
    Ok(LastRewards {
        epoch: codec.get_u32()?,
        reward: Value(codec.get_u64()?),
    })
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
    pub fn last_rewards_pack_unpack_bijection() -> Result<(), std::io::Error> {
        use std::io::Cursor;

        let last_rewards = LastRewards {
            epoch: 0,
            reward: Value(1),
        };

        let mut c: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let mut codec = Codec::new(c);
        pack_last_rewards(&last_rewards,&mut codec)?;
        c = codec.into_inner();
        c.set_position(0);
        codec = Codec::new(c);
        let deserialize_last_rewards = unpack_last_rewards(&mut codec)?;
        assert_eq!(last_rewards, deserialize_last_rewards);
        Ok(())
    }
}
