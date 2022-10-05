use crate::ledger::Error;
use crate::value::Value;

/// An amount of value owned by the treasury.
///
/// Right now, it doesn't have any mechanism to
/// withdraw money from, so it serves just to
/// record a monotically increasing special account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Treasury(Value);

impl Treasury {
    /// Create a treasury with an initial value
    pub fn initial(v: Value) -> Self {
        Self(v)
    }

    pub fn draw(&mut self, value: Value) -> Value {
        let to_draw = std::cmp::min(value, self.0);
        (self.0).0 -= to_draw.0;
        to_draw
    }

    /// Add some value in the treasury
    pub fn add(&mut self, v: Value) -> Result<(), Error> {
        self.0 = (self.0 + v).map_err(|error| Error::PotValueInvalid { error })?;
        Ok(())
    }

    /// remove some values from the treasury
    pub fn sub(&mut self, value: Value) -> Result<(), Error> {
        self.0 = self
            .0
            .checked_sub(value)
            .map_err(|error| Error::PotValueInvalid { error })?;
        Ok(())
    }

    /// Get value in the treasury
    pub fn value(self) -> Value {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Treasury;
    use crate::ledger::Error;
    use crate::value::{Value, ValueError};
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Treasury {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Treasury::initial(Arbitrary::arbitrary(g))
        }
    }

    #[test]
    pub fn draw_from_treasure() {
        let mut treasury = Treasury::initial(Value(100));
        assert_eq!(treasury.draw(Value(60)), Value(60));
        assert_eq!(treasury.value(), Value(40));

        assert_eq!(treasury.draw(Value(50)), Value(40));
        assert_eq!(treasury.value(), Value::zero());
    }

    #[test]
    pub fn sub_from_treasure() {
        let mut treasury = Treasury::initial(Value(100));
        assert!(treasury.sub(Value(60)).is_ok());
        assert_eq!(treasury.value(), Value(40));

        assert_eq!(treasury.value(), Value(40));

        assert_eq!(
            treasury.sub(Value(50)),
            Err(Error::PotValueInvalid {
                error: ValueError::NegativeAmount
            })
        );
    }
}
