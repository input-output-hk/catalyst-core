use core::ops::Range;
use thiserror::Error;

/// error that may occur when creating a new `Options` using
/// the `new_length` function.
///
/// This function will mark all `Options` with a length of `0` options
/// as invalid.
#[derive(Debug, Error)]
#[error("Invalid multi choice option {num_choices}")]
pub struct InvalidOptionsLength {
    num_choices: u8,
}

/// options for the vote
///
/// currently this is a 4bits structure, allowing up to 16 choices
/// however we may allow more complex object to be set in
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    options_range: Range<u8>,
}

/// a choice
///
/// A `Choice` is a representation of a choice that has been made and must
/// be compliant with the `Options`. A way to validate it is with `Options::validate`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Choice(u8);

impl Options {
    const NUM_CHOICES_MAX: u8 = 0b0001_0000;

    /// create a new `Options` with the given number of available choices
    ///
    /// available choices will go from `0` to `num_choices` not included.
    pub fn new_length(num_choices: u8) -> Result<Self, InvalidOptionsLength> {
        if num_choices > 0 && num_choices <= Self::NUM_CHOICES_MAX {
            let options_range = Range {
                start: 0,
                end: num_choices,
            };
            Ok(Self { options_range })
        } else {
            Err(InvalidOptionsLength { num_choices })
        }
    }

    /// get the byte representation of the `Options`
    pub(crate) fn as_byte(&self) -> u8 {
        self.options_range.end
    }

    /// validate the given `Choice` against the available `Options`
    ///
    /// returns `true` if the choice is valid, `false` otherwise. By _valid_
    /// it is meant as in the context of the available `Options`. There is
    /// obviously no wrong choices to make, only lessons to learn.
    pub fn validate(&self, choice: Choice) -> bool {
        self.options_range.contains(&choice.0)
    }

    pub fn choice_range(&self) -> &core::ops::Range<u8> {
        &self.options_range
    }
}

impl Choice {
    pub fn new(choice: u8) -> Self {
        Choice(choice)
    }

    pub fn as_byte(self) -> u8 {
        self.0
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod property {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Options {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::new_length(u8::arbitrary(g)).unwrap_or_else(|_| Options::new_length(1).unwrap())
        }
    }

    impl Arbitrary for Choice {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self::new(u8::arbitrary(g))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn validate_choices<I>(options: &Options, choices: I, expected: bool)
    where
        I: IntoIterator<Item = Choice>,
    {
        for (index, choice) in choices.into_iter().enumerate() {
            if options.validate(choice) != expected {
                panic!(
                    "Choice 0b{choice:08b} ({index}) is not validated properly against 0b{options:08b}",
                    choice = choice.as_byte(),
                    index = index,
                    options = options.as_byte()
                )
            }
        }
    }

    fn test_with_length(num_choices: u8) {
        let options = Options::new_length(num_choices).unwrap();

        validate_choices(&options, (0..num_choices).map(Choice::new), true);

        validate_choices(&options, (num_choices..=u8::MAX).map(Choice::new), false);
    }

    #[test]
    fn check_validations() {
        for length in 1..=Options::NUM_CHOICES_MAX {
            test_with_length(length)
        }
    }

    #[quickcheck]
    pub fn vote_options_max(num_choices: u8) -> TestResult {
        let options = Options::new_length(num_choices);

        if num_choices == 0 || num_choices > Options::NUM_CHOICES_MAX {
            TestResult::from_bool(options.is_err())
        } else {
            let options = options.expect("non `0` options should always be valid");
            TestResult::from_bool(options.as_byte() == num_choices)
        }
    }
}
