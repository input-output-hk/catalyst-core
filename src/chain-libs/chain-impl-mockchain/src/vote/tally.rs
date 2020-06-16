use crate::{
    stake::Stake,
    value::Value,
    vote::{Choice, Options},
};
use thiserror::Error;

/// weight of a vote
///
/// it is often associated to the `stake`. when the tally is counted,
/// each vote will have the associated weight encoded in.
#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Weight(u64);

/// the tally results
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TallyResult {
    results: Box<[Weight]>,

    options: Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tally {
    Public { result: TallyResult },
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum TallyError {
    #[error("Invalid option choice")]
    InvalidChoice { options: Options, choice: Choice },
}

impl Weight {
    fn is_zero(self) -> bool {
        self.0 == 0
    }

    fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Tally {
    pub fn new_public(result: TallyResult) -> Self {
        Self::Public { result }
    }

    pub fn is_public(&self) -> bool {
        self.public().is_some()
    }

    pub fn public(&self) -> Option<&TallyResult> {
        match self {
            Self::Public { result } => Some(result),
        }
    }
}

impl TallyResult {
    pub fn new(options: Options) -> Self {
        let results = Vec::with_capacity(options.choice_range().len()).into();
        Self { results, options }
    }

    pub fn results(&self) -> &[Weight] {
        &self.results
    }

    pub fn participation(&self) -> Stake {
        let s: u64 = self.results.iter().map(|w| w.0).sum();
        Stake::from_value(Value(s))
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    /// add a vote and its weight on the tally
    ///
    /// if the vote's weight is null (`0`), nothing will be changed.
    ///
    /// # Errors
    ///
    /// The function will fail if the `choice` is not a valid `Option`
    pub fn add_vote<W>(&mut self, choice: Choice, weight: W) -> Result<(), TallyError>
    where
        W: Into<Weight>,
    {
        let weight = weight.into();

        if !self.options.validate(choice) {
            Err(TallyError::InvalidChoice {
                options: self.options.clone(),
                choice,
            })
        } else if weight.is_zero() {
            // we simply ignore the case where the `weight` is nul
            //
            // this may have been just as good as to not do the check as we would have
            // add `0` to the results. However just so we know this is handled
            // properly we know that adding a weight of `0` is ignored
            Ok(())
        } else {
            let index = choice.as_byte() as usize;

            self.results[index].saturating_add(weight);

            Ok(())
        }
    }
}

impl From<Stake> for Weight {
    fn from(stake: Stake) -> Self {
        Self(stake.into())
    }
}

impl From<Value> for Weight {
    fn from(value: Value) -> Self {
        Self(value.0)
    }
}

impl From<u64> for Weight {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<Weight> for u64 {
    fn from(w: Weight) -> Self {
        w.0
    }
}
