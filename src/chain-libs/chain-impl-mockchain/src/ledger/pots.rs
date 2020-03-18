use crate::ledger::Error;
use crate::treasury::Treasury;
use crate::value::{Value, ValueError};
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;
use std::cmp;
use std::fmt::Debug;

/// Special pots of money
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pots {
    pub(crate) fees: Value,
    pub(crate) treasury: Treasury,
    pub(crate) rewards: Value,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Entry {
    Fees(Value),
    Treasury(Value),
    Rewards(Value),
}

#[derive(Debug, Clone, Copy)]
pub enum EntryType {
    Fees,
    Treasury,
    Rewards,
}



fn pack_pot_entry<W: std::io::Write>(entry: &Entry, codec: &mut Codec<W>) -> Result<(), std::io::Error> {
    match entry {
        Entry::Fees(value) => {
            codec.put_u8(0)?;
            codec.put_u64(value.0)?;
        }
        Entry::Treasury(value) => {
            codec.put_u8(1)?;
            codec.put_u64(value.0)?;
        }
        Entry::Rewards(value) => {
            codec.put_u8(2)?;
            codec.put_u64(value.0)?;
        }
    }
    Ok(())
}


fn unpack_pot_entry<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<Entry, std::io::Error> {
    match codec.get_u8()? {
        0 => Ok(Entry::Fees(Value(codec.get_u64()?))),
        1 => Ok(Entry::Treasury(Value(codec.get_u64()?))),
        2 => Ok(Entry::Rewards(Value(codec.get_u64()?))),
        code => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid Entry type code {}", code),
        )),
    }
}


impl Entry {
    pub fn value(&self) -> Value {
        match self {
            Entry::Fees(v) => *v,
            Entry::Treasury(v) => *v,
            Entry::Rewards(v) => *v,
        }
    }

    pub fn entry_type(&self) -> EntryType {
        match self {
            Entry::Fees(_) => EntryType::Fees,
            Entry::Treasury(_) => EntryType::Treasury,
            Entry::Rewards(_) => EntryType::Rewards,
        }
    }
}

pub enum IterState {
    Fees,
    Treasury,
    Rewards,
    Done,
}

pub struct Entries<'a> {
    pots: &'a Pots,
    it: IterState,
}

pub struct Values<'a>(Entries<'a>);

impl<'a> Iterator for Entries<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it {
            IterState::Fees => {
                self.it = IterState::Treasury;
                Some(Entry::Fees(self.pots.fees))
            }
            IterState::Treasury => {
                self.it = IterState::Rewards;
                Some(Entry::Treasury(self.pots.treasury.value()))
            }
            IterState::Rewards => {
                self.it = IterState::Done;
                Some(Entry::Rewards(self.pots.rewards))
            }
            IterState::Done => None,
        }
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            None => None,
            Some(e) => Some(e.value()),
        }
    }
}

impl Pots {
    /// Create a new empty set of pots
    pub fn zero() -> Self {
        Pots {
            fees: Value::zero(),
            treasury: Treasury::initial(Value::zero()),
            rewards: Value::zero(),
        }
    }

    pub fn entries<'a>(&'a self) -> Entries<'a> {
        Entries {
            pots: self,
            it: IterState::Fees,
        }
    }

    pub fn values<'a>(&'a self) -> Values<'a> {
        Values(self.entries())
    }

    /// Sum the total values in the pots
    pub fn total_value(&self) -> Result<Value, ValueError> {
        Value::sum(self.values())
    }

    /// Append some fees in the pots
    pub fn append_fees(&mut self, fees: Value) -> Result<(), Error> {
        self.fees = (self.fees + fees).map_err(|error| Error::PotValueInvalid { error })?;
        Ok(())
    }

    /// Draw rewards from the pot
    #[must_use]
    pub fn draw_reward(&mut self, expected_reward: Value) -> Value {
        let to_draw = cmp::min(self.rewards, expected_reward);
        self.rewards = (self.rewards - to_draw).unwrap();
        to_draw
    }

    /// Siphon all the fees
    #[must_use]
    pub fn siphon_fees(&mut self) -> Value {
        let siphoned = self.fees;
        self.fees = Value::zero();
        siphoned
    }

    /// Add to treasury
    pub fn treasury_add(&mut self, value: Value) -> Result<(), Error> {
        self.treasury.add(value)
    }

    /// Get the value in the treasury
    pub fn fees_value(&self) -> Value {
        self.fees
    }

    /// Get the value in the treasury
    pub fn treasury_value(&self) -> Value {
        self.treasury.value()
    }

    pub fn set_from_entry(&mut self, e: &Entry) {
        match e {
            Entry::Fees(v) => self.fees = *v,
            Entry::Treasury(v) => self.treasury = Treasury::initial(*v),
            Entry::Rewards(v) => self.rewards = *v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use chain_core::property::testing::serialization_bijection;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;
    use std::path::Component::CurDir;

    impl Arbitrary for Pots {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Pots {
                fees: Arbitrary::arbitrary(g),
                treasury: Arbitrary::arbitrary(g),
                rewards: Arbitrary::arbitrary(g),
            }
        }
    }

    #[test]
    pub fn zero_pots() {
        let pots = Pots::zero();
        assert_eq!(pots.fees, Value::zero());
        assert_eq!(pots.treasury, Treasury::initial(Value::zero()));
        assert_eq!(pots.rewards, Value::zero());
    }

    #[quickcheck]
    pub fn entries(pots: Pots) -> TestResult {
        for item in pots.entries() {
            match item {
                Entry::Fees(fees) => {
                    assert_eq!(pots.fees, fees);
                }
                Entry::Treasury(treasury) => {
                    assert_eq!(pots.treasury.value(), treasury);
                }
                Entry::Rewards(rewards) => {
                    assert_eq!(pots.rewards, rewards);
                }
            }
        }
        TestResult::passed()
    }

    #[quickcheck]
    pub fn append_fees(mut pots: Pots, value: Value) -> TestResult {
        if (value + pots.fees).is_err() {
            return TestResult::discard();
        }
        let before = pots.fees;
        pots.append_fees(value).unwrap();
        TestResult::from_bool((before + value).unwrap() == pots.fees)
    }

    #[quickcheck]
    pub fn siphon_fees(mut pots: Pots) -> TestResult {
        let before_siphon = pots.fees;
        let siphoned = pots.siphon_fees();
        if siphoned != before_siphon {
            TestResult::error(format!("{} is not equal to {}", siphoned, before_siphon));
        }
        TestResult::from_bool(pots.fees == Value::zero())
    }

    #[quickcheck]
    pub fn draw_reward(mut pots: Pots, expected_reward: Value) -> TestResult {
        if (expected_reward + pots.rewards).is_err() {
            return TestResult::discard();
        }

        let before_reward = pots.rewards;
        let to_draw = pots.draw_reward(expected_reward);
        let draw_reward = cmp::min(before_reward, expected_reward);
        if to_draw != draw_reward {
            TestResult::error(format!(
                "{} is not equal to smallest of pair({},{})",
                to_draw, before_reward, expected_reward
            ));
        }
        TestResult::from_bool(pots.rewards == (before_reward - to_draw).unwrap())
    }

    #[quickcheck]
    pub fn treasury_add(mut pots: Pots, value: Value) -> TestResult {
        if (value + pots.rewards).is_err() {
            return TestResult::discard();
        }
        let before_add = pots.treasury.value();
        pots.treasury_add(value).unwrap();
        TestResult::from_bool(pots.treasury.value() == (before_add + value).unwrap())
    }

    #[test]
    fn entry_pack_unpack_bijection() -> Result<(), std::io::Error> {
        use std::io::Cursor;
        for entry_value in [
            Entry::Fees(Value(10)),
            Entry::Rewards(Value(10)),
            Entry::Treasury(Value(10)),
        ]
        .iter()
        {
            let mut c: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            let mut codec = Codec::new(c);
            pack_pot_entry(entry_value, &codec)?;
            c= codec.into_inner();
            c.set_position(0);
            codec = Codec::new(c);
            let other_value = unpack_pot_entry(&mut c)?;
            assert_eq!(entry_value, &other_value);
        }
        Ok(())
    }
}
