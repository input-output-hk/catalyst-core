use super::*;
use rayon::prelude::*;
use std::collections::HashMap;

// make steps asymmetric, in order to better use caching of baby steps.
// balance of 2 means that baby steps are 2 time more than sqrt(max_votes)
const DEFAULT_BALANCE: u64 = 2;

/// Holds precomputed baby steps for the baby-stap giant-step algorithm
/// for solving discrete log on ECC
#[derive(Debug, Clone)]
pub struct BabyStepsTable {
    table: HashMap<Option<[u8; Coordinate::BYTES_LEN]>, u64>,
    baby_step_size: u64,
    giant_step: GroupElement,
}

impl BabyStepsTable {
    /// Generate the table with asymmetrical steps,
    /// optimized for multiple reuse of the same table.
    pub fn generate(max_value: u64) -> Self {
        Self::generate_with_balance(max_value, DEFAULT_BALANCE)
    }

    /// Generate the table with the given balance. Balance is used to make
    /// steps asymmetrical. If the table is reused multiple times with the same
    /// max_value it is recommended to set a balance > 1, since this will
    /// allow to cache more results, at the expense of a higher memory footprint.
    ///
    /// For example, a balance of 2 means that the table will precompute 2 times more
    /// baby steps than the standard O(sqrt(n)), 1 means symmetrical steps.
    pub fn generate_with_balance(max_value: u64, balance: u64) -> Self {
        let sqrt_step_size = (max_value as f64).sqrt().ceil() as u64;
        let baby_step_size = sqrt_step_size * balance;
        let mut bs = HashMap::new();
        let gen = GroupElement::generator();
        let mut e = GroupElement::zero();
        // With ECC we can use the property that P and -P share a coordinate
        for i in 0..=baby_step_size / 2 {
            bs.insert(e.compress().map(|(c, _sign)| c.to_bytes()), i);
            e = &e + &gen;
        }
        assert!(!bs.is_empty());
        assert!(baby_step_size > 0);
        Self {
            table: bs,
            baby_step_size,
            giant_step: GroupElement::generator() * Scalar::from_u64(baby_step_size).negate(),
        }
    }
}

#[derive(Debug)]
pub struct MaxLogExceeded;

// Solve the discrete log on ECC using baby step giant step algorithm
pub fn baby_step_giant_step(
    points: Vec<GroupElement>,
    max_log: u64,
    table: &BabyStepsTable,
) -> Result<Vec<u64>, MaxLogExceeded> {
    let baby_step_size = table.baby_step_size;
    let giant_step = &table.giant_step;
    let table = &table.table;
    points
        .into_par_iter()
        .map(|mut p| {
            let mut a = 0;
            loop {
                if let Some(x) = table.get(&p.compress().map(|(c, _sign)| c.to_bytes())) {
                    let r = if Scalar::from_u64(*x) * GroupElement::generator() == p {
                        a * baby_step_size + x
                    } else {
                        a * baby_step_size - x
                    };
                    return Ok(r);
                }
                if a * baby_step_size > max_log {
                    return Err(MaxLogExceeded);
                }
                p = p + giant_step;
                a += 1;
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use smoke::{
        generator::{self, BoxGenerator},
        Generator,
    };

    #[test]
    #[should_panic]
    fn table_not_empty() {
        let _ = BabyStepsTable::generate_with_balance(0, 1);
    }

    #[test]
    fn quick() {
        let table = BabyStepsTable::generate_with_balance(25, 1);
        let p = GroupElement::generator();
        let votes = (0..100).collect::<Vec<_>>();
        let points = votes
            .iter()
            .map(|k| {
                #[cfg(not(feature = "zerocaf"))]
                let ks = Scalar::from_u64(*k);
                #[cfg(feature = "zerocaf")]
                let ks = FieldElement::from_u64(*k);
                &p * ks
            })
            .collect();
        let results = baby_step_giant_step(points, 100, &table).unwrap();
        assert_eq!(votes, results);
    }

    fn fe_vec_generator() -> BoxGenerator<[(GroupElement, u64); 64]> {
        generator::Array64::new(generator::num::<u16>().map(|a| {
            (
                GroupElement::generator() * Scalar::from_u64(a as u64),
                a as u64,
            )
        }))
        .into_boxed()
    }

    fn table_generator_default() -> BoxGenerator<BabyStepsTable> {
        generator::range::<u16>(1..u16::MAX)
            .map(|a| BabyStepsTable::generate(a as u64))
            .into_boxed()
    }

    fn table_generator_with_balance() -> BoxGenerator<BabyStepsTable> {
        generator::range::<u16>(1..u16::MAX)
            .and(generator::range::<u8>(1..u8::MAX))
            .map(|(n, b)| BabyStepsTable::generate_with_balance(n as u64, b as u64))
            .into_boxed()
    }

    #[test]
    #[ignore]
    fn bsgs_correctness() {
        use smoke::{forall, property, run, Testable};
        run(|ctx| {
            forall(fe_vec_generator().and(table_generator_default()))
                .ensure(|(points, table)| {
                    let (points, ks): (Vec<_>, Vec<_>) = points.to_vec().into_iter().unzip();
                    let b = property::equal(
                        ks,
                        baby_step_giant_step(points.to_vec(), u16::MAX as u64, &table).unwrap(),
                    );
                    b
                })
                .test(ctx);
            forall(fe_vec_generator().and(table_generator_with_balance()))
                .ensure(|(points, table)| {
                    let (points, ks): (Vec<_>, Vec<_>) = points.to_vec().into_iter().unzip();
                    property::equal(
                        ks,
                        baby_step_giant_step(points.to_vec(), u16::MAX as u64, &table).unwrap(),
                    )
                })
                .test(ctx);
        });
    }
}
