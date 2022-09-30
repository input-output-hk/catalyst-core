use super::{Error, SnapshotInfo};
use fraction::{BigFraction, Fraction};
use rust_decimal::prelude::ToPrimitive;

/// Calculates ceil(a / b) where a and b are integers with perfect precision
#[inline]
fn int_ceil(a: u64, b: u64) -> u64 {
    assert!(b > 0);
    (a + b - 1) / b
}

/// Let's denote with X the current voting power of user U and with TOT the total voting power
/// and suppose X / TOT >= threshold.
/// We want to remove a quantity Y from X such that (X - Y)/ (TOT - Y) <= treshold.
/// The solution to this equation is therefore Y <= (X - T*TOT) / (1 - T).
/// Since we work with integer numbers, we will use Y' = ceil(Y).
///
/// Panics if threshold is > 1 or x/tot <= threshold
fn calc_vp_to_remove(x: u64, tot: u64, threshold: Fraction) -> u64 {
    assert!(threshold <= Fraction::from(1u64));
    assert!(threshold < Fraction::new(x, tot));
    // Avoid cluttering this part with overflow checks by using big ints.
    // The result is guaranteed to fit in a u64 anyway.
    let threshold = BigFraction::from_fraction(threshold);
    let num = BigFraction::from(x) - threshold.clone() * BigFraction::from(tot);
    let denum = BigFraction::from(1u64) - threshold;
    // Y must be smaller than X, hence it's safe to unwrap here
    (num / denum).ceil().to_u64().unwrap()
}

/// Cap each individual voting power according to the threshold, if possible.
///
/// Obviously, to cap each individual's influence to T, we need at east M = ceil(1/T) participants.
/// If we have M or more participants, it's always possible to do it.
///
/// The algorithm is as follows:
/// 1. Sort the list of voters V = [V_0, ..., V_n] in non-decreasing order according to voting power Vp_i.
/// 2. Let S be the set of contiguous voters being currently considered, X the voting power of each of the voters inside S,
///   and V_l the leftmost voter inside S. All voters in S will be given the same voting power.
///   Initially, set S = {V_n}, X = Vp_n.
///   2.1 Calculate Y, how much voting power we should remove from each of the voters in S to match the
///       threshold (see [calc_vp_to_remove] for more information).
///       If (a) X - Y >= Vp_{l-1}, set X to X - Y. The current list is the solution.
///       If (b) X - Y < Vp_{l-1}, set X to Vp_{l-1}, add V_{l-1} to S and repeat from step 2.1.
///       It's easy to see we need to repeat step 2.1 at most min(ceil(1/T), N) times.
///
/// Complexity: O(NlogN + min(ceil(1/T), N))
pub fn cap_voting_influence(
    mut voters: Vec<SnapshotInfo>,
    threshold: Fraction,
) -> Result<Vec<SnapshotInfo>, Error> {
    voters = voters
        .into_iter()
        .filter(|v| v.hir.voting_power > 0.into())
        .collect();

    if voters.is_empty() {
        return Ok(voters);
    }

    if Fraction::new(1u64, voters.len() as u64) > threshold {
        return Err(Error::NotEnoughVoters);
    }
    let mut tot = 0u64;
    // can't check for overflows with Iterator::sum()
    for v in &voters {
        tot = tot
            .checked_add(u64::from(v.hir.voting_power))
            .ok_or(Error::Overflow)?;
    }
    voters.sort_unstable_by_key(|v| v.hir.voting_power);
    let last = voters.len() - 1;

    let last_vp = u64::from(voters[last].hir.voting_power);
    if Fraction::new(last_vp, tot) <= threshold {
        return Ok(voters);
    }

    let mut prev = last - 1;
    let mut next_vp = last_vp;
    let next_vp = loop {
        let set_len = (last - prev) as u64;
        // Since it's guaranteed that all elements in S will have the same voting power,
        // to calculate the value of Y for each element in S it's sufficient to treat it as a single
        // value which should not exceed a threshold of T * |S| and get Y'.
        // Given Y', we need to 'spread' it among all elements, from which we get Y = ceil(Y' / len).
        let y = int_ceil(
            calc_vp_to_remove(next_vp * set_len, tot, threshold * set_len.into()),
            set_len,
        );
        let prev_vp = u64::from(voters[prev].hir.voting_power);
        if next_vp - y >= prev_vp {
            // Case (a): we can fix the current voter without impacting other
            next_vp -= y;
            break next_vp;
        } else {
            // Case (b): we need to enlarge the set S and try again
            tot -= (next_vp - prev_vp) * set_len;
            next_vp = prev_vp;
        }
        assert!(prev > 0);
        prev -= 1;
    };

    for v in &mut voters[prev + 1..=last] {
        v.hir.voting_power = next_vp.into();
    }

    Ok(voters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{voter_hir::tests::VpRange, VoterHIR};
    use proptest::{collection::vec, prelude::*};
    use test_strategy::proptest;

    const DEFAULT_VP_STRATEGY: VpRange = VpRange::ada_distribution();

    #[proptest]
    fn test_insufficient_voters(
        #[strategy(vec(any::<SnapshotInfo>(), 1..100))] voters: Vec<SnapshotInfo>,
    ) {
        let cap = Fraction::new(1u64, voters.len() as u64 + 1);
        assert!(cap_voting_influence(voters, cap).is_err());
    }

    #[proptest]
    fn test_exact_voters(
        #[strategy(vec(any_with::<SnapshotInfo>((Default::default(), DEFAULT_VP_STRATEGY)), 1..100))]
        voters: Vec<SnapshotInfo>,
    ) {
        let cap = Fraction::new(1u64, voters.len() as u64);
        let min = voters.iter().map(|v| v.hir.voting_power).min().unwrap();
        let res = cap_voting_influence(voters, cap).unwrap();
        assert!(res
            .iter()
            .map(|entry| entry.hir.voting_power)
            .all(|vp| vp == min));
    }

    #[proptest]
    fn test_exact_voters_fixed_at_threshold(
        #[strategy(vec(any_with::<SnapshotInfo>((Default::default(), DEFAULT_VP_STRATEGY)), 101..=101))]
        voters: Vec<SnapshotInfo>,
    ) {
        let cap = Fraction::new(999u64, 100000u64);
        let res = cap_voting_influence(voters, cap).unwrap();
        let vps = res
            .iter()
            .map(|entry| u64::from(entry.hir.voting_power))
            .collect::<Vec<_>>();
        let tot = vps.iter().sum::<u64>();
        for v in vps {
            assert!(Fraction::new(v, tot) <= cap);
        }
    }

    #[proptest]
    fn test_exact_voters_fixed_below_threshold(
        #[strategy(vec(any_with::<SnapshotInfo>((Default::default(), DEFAULT_VP_STRATEGY)), 100..=100))]
        voters: Vec<SnapshotInfo>,
    ) {
        let cap = Fraction::new(9u64, 1000u64);
        assert!(cap_voting_influence(voters, cap).is_err());
    }

    #[proptest]
    fn test_below_threshold(
        #[strategy(vec(any_with::<SnapshotInfo>((Default::default(), DEFAULT_VP_STRATEGY)), 100..300))]
        voters: Vec<SnapshotInfo>,
    ) {
        let cap = Fraction::new(1u64, 100u64);
        let res = cap_voting_influence(voters, cap).unwrap();
        let vps = res
            .iter()
            .map(|entry| u64::from(entry.hir.voting_power))
            .collect::<Vec<_>>();
        let tot = vps.iter().sum::<u64>();
        for v in vps {
            assert!(Fraction::new(v, tot) <= cap);
        }
    }

    impl Arbitrary for SnapshotInfo {
        type Parameters = (String, VpRange);
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any_with::<VoterHIR>(args)
                .prop_map(|hir| Self {
                    contributions: Vec::new(),
                    hir,
                })
                .boxed()
        }
    }
}
