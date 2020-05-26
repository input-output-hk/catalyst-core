use crate::date::Epoch;
use crate::stake::Stake;
use crate::value::{Value, ValueError};
use chain_core::mempack::{ReadBuf, ReadError};
use std::num::{NonZeroU32, NonZeroU64};
use typed_bytes::ByteBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundingType {
    Linear,
    Halvening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: u64,
    pub denominator: NonZeroU64,
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.denominator == other.denominator {
            self.numerator.cmp(&other.numerator)
        } else {
            let left = self.numerator as u128 * other.denominator.get() as u128;
            let right = other.numerator as u128 * self.denominator.get() as u128;
            left.cmp(&right)
        }
    }
}

impl Ratio {
    pub fn zero() -> Self {
        Ratio {
            numerator: 0,
            denominator: NonZeroU64::new(1).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaxType {
    // what get subtracted as fixed value
    pub fixed: Value,
    // Ratio of tax after fixed amout subtracted
    pub ratio: Ratio,
    // Max limit of tax
    pub max_limit: Option<NonZeroU64>,
}

impl TaxType {
    pub fn zero() -> Self {
        TaxType {
            fixed: Value(0),
            ratio: Ratio::zero(),
            max_limit: None,
        }
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.u64(self.fixed.0)
            .u64(self.ratio.numerator)
            .u64(self.ratio.denominator.get())
            .u64(self.max_limit.map_or(0, |v| v.get()))
    }

    pub fn read_frombuf(rb: &mut ReadBuf) -> Result<Self, ReadError> {
        let fixed = rb.get_u64().map(Value)?;
        let num = rb.get_u64()?;
        let denom = rb.get_u64()?;
        let limit = rb.get_u64()?;
        let denominator = NonZeroU64::new(denom).map_or_else(
            || {
                Err(ReadError::StructureInvalid(
                    "ratio fraction divisor invalid".to_string(),
                ))
            },
            Ok,
        )?;
        if num > denom {
            return Err(ReadError::StructureInvalid(
                "ratio fraction invalid bigger than 1".to_string(),
            ));
        }

        Ok(TaxType {
            fixed,
            ratio: Ratio {
                numerator: num,
                denominator,
            },
            max_limit: NonZeroU64::new(limit),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Limit {
    /// the drawn value will not be limited
    None,

    /// The drawn value will be limited by the absoluted stake in the system
    /// with a given ratio.
    ByStakeAbsolute(Ratio),
}

/// Parameters for rewards calculation. This controls:
///
/// * Rewards contributions
/// * Treasury cuts
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Parameters {
    /// This is an initial_value for the linear or halvening function.
    /// In the case of the linear function it is the value that is going to be calculated
    /// from the contribution.
    pub initial_value: u64,
    /// This is the ratio used by either the linear or the halvening function.
    /// The semantic and result of this is deeply linked to either.
    pub compounding_ratio: Ratio,
    /// The type of compounding
    pub compounding_type: CompoundingType,
    /// Number of epoch between reduction phase, cannot be 0
    pub epoch_rate: NonZeroU32,
    /// When to start
    pub epoch_start: Epoch,
    /// Max Drawing limit
    pub reward_drawing_limit_max: Limit,
    /// Pool Capping
    /// This doesn't really make sense
    pub pool_participation_capping: Option<(NonZeroU32, NonZeroU32)>,
}

impl Parameters {
    pub fn zero() -> Self {
        Parameters {
            initial_value: 0,
            compounding_ratio: Ratio::zero(),
            compounding_type: CompoundingType::Linear,
            epoch_rate: NonZeroU32::new(u32::max_value()).unwrap(),
            epoch_start: 0,
            reward_drawing_limit_max: Limit::None,
            pool_participation_capping: None,
        }
    }
}

/// A value distributed between tax and remaining
#[derive(Debug, Clone)]
pub struct TaxDistribution {
    pub taxed: Value,
    pub after_tax: Value,
}

#[derive(Debug, Clone)]
pub struct SystemInformation {
    pub declared_stake: Stake,
}

/// Calculate the reward contribution from the parameters
///
/// Note that the contribution in the system is still bounded by the remaining
/// rewards pot, which is not taken in considering for this calculation.
pub fn rewards_contribution_calculation(
    epoch: Epoch,
    params: &Parameters,
    system_info: &SystemInformation,
) -> Value {
    assert!(params.epoch_rate.get() != 0);

    if epoch < params.epoch_start {
        return Value::zero();
    }

    let zone = ((epoch - params.epoch_start) / params.epoch_rate.get()) as u64;
    let drawn = match params.compounding_type {
        CompoundingType::Linear => {
            // C - rratio * (#epoch / erate)
            let rr = &params.compounding_ratio;
            let reduce_by = (rr.numerator * zone) / rr.denominator.get();
            if params.initial_value >= reduce_by {
                Value(params.initial_value - reduce_by)
            } else {
                Value::zero()
            }
        }
        CompoundingType::Halvening => {
            // mathematical formula is : C * rratio ^ (#epoch / erate)
            // although we perform it as a for loop, with the rationale
            // that it allow for integer computation and that the reduce_epoch_rate
            // should prevent growth to large amount of zones
            let rr = &params.compounding_ratio;
            const SCALE: u128 = 1_000_000_000_000_000_000;

            let mut acc = params.initial_value as u128 * SCALE;
            for _ in 0..zone {
                acc *= rr.numerator as u128;
                acc /= rr.denominator.get() as u128;
            }

            Value((acc / SCALE) as u64)
        }
    };

    match params.reward_drawing_limit_max {
        Limit::None => drawn,
        Limit::ByStakeAbsolute(ratio) => {
            let x = (u64::from(system_info.declared_stake) as u128 * ratio.numerator as u128)
                / ratio.denominator.get() as u128;
            std::cmp::min(drawn, Value(x as u64))
        }
    }
}

/// Tax some value into the tax value and what is remaining
pub fn tax_cut(v: Value, tax_type: &TaxType) -> Result<TaxDistribution, ValueError> {
    let mut left = v;
    let mut taxed = Value::zero();

    // subtract fix amount
    match left - tax_type.fixed {
        Ok(left1) => {
            left = left1;
            taxed = (taxed + tax_type.fixed)?;
        }
        Err(_) => {
            return Ok(TaxDistribution {
                taxed: v,
                after_tax: Value::zero(),
            })
        }
    };

    // calculate and subtract ratio
    {
        let rr = tax_type.ratio;
        let olimit = tax_type.max_limit;

        const SCALE: u128 = 1_000_000_000;
        let out = ((((left.0 as u128 * SCALE) * rr.numerator as u128)
            / rr.denominator.get() as u128)
            / SCALE) as u64;
        let treasury_cut = match olimit {
            None => Value(out),
            Some(limit) => Value(std::cmp::min(limit.get(), out)),
        };

        match left - treasury_cut {
            Ok(left2) => {
                left = left2;
                taxed = (taxed + treasury_cut)?;
            }
            Err(_) => {
                left = Value::zero();
                taxed = (taxed + left)?;
            }
        }
    };

    Ok(TaxDistribution {
        taxed,
        after_tax: left,
    })
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn tax_cut_fully_accounted(v: Value, treasury_tax: TaxType) -> TestResult {
        match tax_cut(v, &treasury_tax) {
            Ok(td) => {
                let sum = (td.taxed + td.after_tax).unwrap();
                if sum == v {
                    TestResult::passed()
                } else {
                    TestResult::error(format!(
                        "mismatch taxed={} remaining={} expected={} got={} for {:?}",
                        td.taxed, td.after_tax, v, sum, treasury_tax
                    ))
                }
            }
            Err(_) => TestResult::discard(),
        }
    }

    #[test]
    fn ratio_cmp_works() {
        use std::cmp::Ordering;
        let r1 = Ratio {
            numerator: 10,
            denominator: NonZeroU64::new(20).unwrap(),
        };
        let r2 = Ratio {
            numerator: 20,
            denominator: NonZeroU64::new(10).unwrap(),
        };
        let r3 = Ratio {
            numerator: 20,
            denominator: NonZeroU64::new(40).unwrap(),
        };
        assert_eq!(r1.cmp(&r2), Ordering::Less);
        assert_eq!(r2.cmp(&r1), Ordering::Greater);
        assert_eq!(r1.cmp(&r3), Ordering::Equal);
        assert_eq!(r3.cmp(&r1), Ordering::Equal);
    }

    #[test]
    fn rewards_contribution_calculation_epoch_start_smaller_than_epoch() {
        let mut params = Parameters::zero();
        params.epoch_start = 1;
        let epoch = 0;
        let system_info = SystemInformation {
            declared_stake: Stake::from_value(Value(100)),
        };
        assert_eq!(
            rewards_contribution_calculation(epoch, &params, &system_info),
            Value::zero()
        );
    }

    #[test]
    fn rewards_contribution_calculation_initial_value_smaller_than_reduce_by() {
        let params = Parameters {
            initial_value: 9,
            compounding_ratio: Ratio {
                numerator: 100,
                denominator: NonZeroU64::new(10).unwrap(),
            },
            compounding_type: CompoundingType::Linear,
            epoch_rate: NonZeroU32::new(1).unwrap(),
            epoch_start: 0,
            reward_drawing_limit_max: Limit::None,
            pool_participation_capping: None,
        };
        let epoch = 1;
        let system_info = SystemInformation {
            declared_stake: Stake::from_value(Value(100)),
        };
        assert_eq!(
            rewards_contribution_calculation(epoch, &params, &system_info),
            Value::zero()
        );
    }

    impl Arbitrary for TaxType {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let fixed = Arbitrary::arbitrary(gen);
            let denominator = u64::arbitrary(gen) + 1;
            let numerator = u64::arbitrary(gen) % denominator;
            let max_limit = NonZeroU64::new(u64::arbitrary(gen));

            TaxType {
                fixed,
                ratio: Ratio {
                    numerator,
                    denominator: NonZeroU64::new(denominator).unwrap(),
                },
                max_limit,
            }
        }
    }

    impl Arbitrary for Limit {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if bool::arbitrary(g) {
                Limit::None
            } else {
                Limit::ByStakeAbsolute(Ratio::arbitrary(g))
            }
        }
    }

    impl Arbitrary for Parameters {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let epoch_rate = {
                let mut number = u32::arbitrary(g);
                if number == 0 {
                    number += 1;
                }
                NonZeroU32::new(number).unwrap()
            };

            Parameters {
                initial_value: u64::arbitrary(g),
                compounding_ratio: Ratio::arbitrary(g),
                compounding_type: CompoundingType::arbitrary(g),
                epoch_rate,
                epoch_start: Arbitrary::arbitrary(g),
                reward_drawing_limit_max: Limit::arbitrary(g),
                pool_participation_capping: None,
            }
        }
    }

    impl Arbitrary for CompoundingType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let option: u8 = u8::arbitrary(g) % 2;
            match option {
                0 => CompoundingType::Linear,
                2 => CompoundingType::Halvening,
                _ => unreachable!(),
            }
        }
    }
}
