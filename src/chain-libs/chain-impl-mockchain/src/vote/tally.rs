use crate::{
    stake::Stake,
    value::Value,
    vote::{Choice, Options},
};
use chain_vote::EncryptedTally;
use std::fmt;
use thiserror::Error;

use super::VoteError;

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tally {
    Public { result: TallyResult },
    Private { state: PrivateTallyState },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrivateTallyState {
    Encrypted { encrypted_tally: EncryptedTally },
    Decrypted { result: TallyResult },
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum TallyError {
    #[error("Invalid option choice")]
    InvalidChoice { options: Options, choice: Choice },
    #[error("Invalid privacy")]
    InvalidPrivacy,
    #[error("this private tally is already decryptred")]
    TallyAlreadyDecrypted,
    #[error("the encrypted tally was not provided yet")]
    NoEncryptedTally,
    #[error("bad decryption share data")]
    BadDecryptShares,
    #[error("invalid decrypted tally")]
    InvalidDecryption,
}

impl Weight {
    fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[must_use = "Does not modify the internal state"]
    fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl Tally {
    pub fn new_public(result: TallyResult) -> Self {
        Self::Public { result }
    }

    pub fn new_private(encrypted_tally: EncryptedTally) -> Self {
        Self::Private {
            state: PrivateTallyState::Encrypted { encrypted_tally },
        }
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public { .. })
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private { .. })
    }

    pub fn result(&self) -> Option<&TallyResult> {
        match self {
            Self::Public { result } => Some(result),
            Self::Private {
                state: PrivateTallyState::Decrypted { result },
            } => Some(result),
            _ => None,
        }
    }

    pub fn private_encrypted(&self) -> Result<&EncryptedTally, TallyError> {
        match self {
            Self::Private {
                state: PrivateTallyState::Encrypted { encrypted_tally },
            } => Ok(encrypted_tally),
            Self::Private {
                state: PrivateTallyState::Decrypted { .. },
            } => Err(TallyError::TallyAlreadyDecrypted),
            Self::Public { .. } => Err(TallyError::InvalidPrivacy),
        }
    }

    pub fn private_set_result(mut self, result: TallyResult) -> Result<Self, TallyError> {
        match &mut self {
            Self::Private { state } => {
                if let PrivateTallyState::Decrypted { .. } = state {
                    return Err(TallyError::TallyAlreadyDecrypted);
                }
                *state = PrivateTallyState::Decrypted { result };
                Ok(self)
            }
            _ => Err(TallyError::InvalidPrivacy),
        }
    }
}

impl TallyResult {
    pub fn new(options: Options) -> Self {
        let len = options.choice_range().len();
        let results = vec![Weight(0); len].into();
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
    pub fn add_vote<W>(&mut self, choice: Choice, weight: W) -> Result<(), VoteError>
    where
        W: Into<Weight>,
    {
        let weight = weight.into();

        if !self.options.validate(choice) {
            Err(VoteError::InvalidChoice {
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

            self.results[index] = self.results[index].saturating_add(weight);

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

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Tally, TallyResult, Weight};
    use crate::{
        stake::Stake,
        vote::{Choice, Options, VoteError},
    };
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for TallyResult {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            TallyResult::new(Arbitrary::arbitrary(g))
        }
    }

    #[test]
    pub fn weight_test() {
        let mut weight = Weight(0);
        let weight_10 = Weight(10);
        assert!(weight.is_zero());

        weight = weight.saturating_add(weight_10);
        assert!(!weight.is_zero());

        let value: u64 = weight.into();
        assert_eq!(value, 10);
    }

    #[test]
    pub fn tally_result_add_vote_invalid_test() {
        let options = Options::new_length(3u8).unwrap();
        let mut tally_result = TallyResult::new(options.clone());
        let choice = Choice::new(4);
        assert_eq!(
            tally_result.add_vote(choice, Weight(1)),
            Err(VoteError::InvalidChoice { options, choice })
        );
    }

    #[test]
    pub fn tally_result_add_zero_weight_vote_test() {
        let options = Options::new_length(3u8).unwrap();
        let mut tally_result = TallyResult::new(options);
        let choice = Choice::new(0);

        let results = tally_result.results().to_vec();

        tally_result.add_vote(choice, Weight(0)).unwrap();
        assert_eq!(tally_result.results().to_vec(), results);
    }

    #[test]
    pub fn tally_result_add_zero_weight_add_correct_vote() {
        let options = Options::new_length(3u8).unwrap();
        let mut tally_result = TallyResult::new(options.clone());
        let choice = Choice::new(2);
        tally_result.add_vote(choice, Weight(1)).unwrap();
        assert_eq!(tally_result.participation(), Stake(1));
        assert_eq!(*tally_result.options(), options);
    }

    #[quickcheck]
    pub fn tally(tally_result: TallyResult) -> TestResult {
        let tally = Tally::new_public(tally_result.clone());
        TestResult::from_bool(tally.is_public() && (*tally.result().unwrap()) == tally_result)
    }
}
