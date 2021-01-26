#[cfg(not(feature = "zerocaf"))]
use super::p256k1::*;
#[cfg(feature = "zerocaf")]
use super::zerocaf::*;
use lazy_static::*;
use std::collections::HashMap;
use std::convert::TryFrom;

// make steps asymmetric, in order to better use caching of baby steps.
// balance of 2 means that baby steps are 2 time more than sqrt(MAX_VOTES_BSGS)
const BALANCE: u64 = 2;
// change this to adjust precomputed range, given this number it is possible to
// find mupltiples up to SQRT_STEP_SIZE ^ 2 - 1
lazy_static! {
    static ref SQRT_STEP_SIZE: u64 = {
        let max_votes = option_env!("MAX_VOTES_BSGS")
            .unwrap_or("31999999999") // 1e10 -1
            .parse::<u64>()
            .expect("Invalid MAX_VOTES_BSGS: expected a integer");
        ((max_votes as f64).sqrt() + 1.0) as u64
    };
    static ref BABY_STEPS_SIZE: u64 = *SQRT_STEP_SIZE * BALANCE;
}

lazy_static! {
    static ref BS: HashMap<[u8; 32], u64> = {
        let mut bs = HashMap::new();
        let gen = GroupElement::generator();
        let mut e = GroupElement::zero();
        // use negation trick and store only the x coordinate
        for i in 0..*BABY_STEPS_SIZE / 2 {
            bs.insert(<[u8; 32]>::try_from(&e.to_bytes()[1..33]).unwrap(), i);
            e = &e + &gen;
        }
        bs
    };
}

#[cfg(not(feature = "zerocaf"))]
lazy_static! {
    static ref GIANT_STEP: GroupElement =
        GroupElement::generator() * Scalar::from_u64(*BABY_STEPS_SIZE).negate();
}

#[cfg(feature = "zerocaf")]
lazy_static! {
    static ref GIANT_STEP: GroupElement =
        GroupElement::generator() * FieldElement::from_u64(*BABY_STEPS_SIZE).negate();
}

// Solve the discrete log on ECC using baby step giant step algorithm
pub fn baby_step_giant_step(points: Vec<GroupElement>, max_votes: u64) -> Vec<Option<u64>> {
    points
        .into_iter()
        .map(|mut p| {
            let mut a = 0;
            loop {
                if let Some(x) = BS.get(&p.to_bytes()[1..33]) {
                    if Scalar::from_u64(*x) * GroupElement::generator() == p {
                        return Some(a * (*BABY_STEPS_SIZE) + x);
                    } else {
                        return Some(a * (*BABY_STEPS_SIZE) - x);
                    }
                }
                // This should not happen if the precomputed range is correctly
                // sized, set MAX_VOTES_BSGS at compile time for best performances
                if a * (*BABY_STEPS_SIZE) + *BABY_STEPS_SIZE - 1 > max_votes {
                    break;
                }
                p = p + &*GIANT_STEP;
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
        let p = GroupElement::generator();
        let votes = [
            (*SQRT_STEP_SIZE).pow(2) - 1,
            (*SQRT_STEP_SIZE).pow(2),
            (*SQRT_STEP_SIZE).pow(2) * 2 - 1,
        ];
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
            baby_step_giant_step(points, (*SQRT_STEP_SIZE).pow(2) * 2 - 1)[..]
        );
    }
}
