use super::p256k1::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;

// make steps asymmetric, in order to better use caching of baby steps.
// balance of 2 means that baby steps are 2 time more than sqrt(MAX_VOTES_BSGS)
const DEFAULT_BALANCE: u64 = 2;

/// Holds precomputed baby steps for the baby-stap giant-step algorithm
/// for solving discrete log on ECC and tally private votes
pub struct PrivateTallyTable {
    table: HashMap<[u8; 32], u64>,
    baby_step_size: u64,
    giant_step: GroupElement,
}

impl PrivateTallyTable {
    /// Generate the table with asymmetrical steps,
    /// optimized for multiple reuse of the same table.
    pub fn generate(max_voting_power: u64) -> Self {
        Self::generate_with_balance(max_voting_power, DEFAULT_BALANCE)
    }

    /// Generate the table with the given balance. Balance is used to make
    /// steps asymmetrical. If the table is reused multiple times with the same
    /// max_voting_power it is recommended to set a balance > 1, since this will
    /// allow to cache more results, at the expense of a higher memory footprint.
    ///
    /// For example, a balance of 2 means that the table will precompute 2 times more
    /// baby steps than the standard O(sqrt(n)), 1 means symmetrical steps.
    pub fn generate_with_balance(max_voting_power: u64, balance: u64) -> Self {
        let sqrt_step_size = (max_voting_power as f64).sqrt().ceil() as u64;
        let baby_step_size = sqrt_step_size * balance;
        let mut bs = HashMap::new();
        let gen = GroupElement::generator();
        let mut e = GroupElement::zero();
        // use negation trick and store only the x coordinate
        for i in 0..=baby_step_size / 2 {
            bs.insert(<[u8; 32]>::try_from(&e.to_bytes()[1..33]).unwrap(), i);
            e = &e + &gen;
        }
        Self {
            table: bs,
            baby_step_size: baby_step_size,
            giant_step: GroupElement::generator() * Scalar::from_u64(baby_step_size).negate(),
        }
    }
}

// Solve the discrete log on ECC using baby step giant step algorithm
pub fn baby_step_giant_step(
    points: Vec<GroupElement>,
    max_votes: u64,
    table: &PrivateTallyTable,
) -> Vec<Option<u64>> {
    let baby_step_size = table.baby_step_size;
    let giant_step = &table.giant_step;
    let table = &table.table;
    points
        .into_par_iter()
        .map(|mut p| {
            let mut a = 0;
            loop {
                if let Some(x) = table.get(&p.to_bytes()[1..33]) {
                    if Scalar::from_u64(*x) * GroupElement::generator() == p {
                        return Some(a * baby_step_size + x);
                    } else {
                        return Some(a * baby_step_size - x);
                    }
                }
                // This should not happen if the precomputed range is correctly
                // sized, set MAX_VOTES_BSGS at compile time for best performances
                if a * baby_step_size > max_votes {
                    break;
                }
                p = p + giant_step;
                a += 1;
            }

            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bsgs() {
        let table = PrivateTallyTable::generate_with_balance(25, 1);
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
        assert_eq!(
            votes.iter().map(|v| Some(*v)).collect::<Vec<_>>(),
            baby_step_giant_step(points, 100, &table)[..]
        );
    }
}
