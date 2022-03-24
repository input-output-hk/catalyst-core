use crate::community_advisors::models::{
    ReviewRanking::{self, *},
    VeteranAdvisorId, VeteranRankingRow,
};
use crate::rewards::Rewards;
use itertools::Itertools;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

#[derive(Serialize)]
pub struct VeteranAdvisorIncentive {
    pub rewards: Rewards,
    pub reputation: u64,
}

pub type VcaRewards = HashMap<VeteranAdvisorId, VeteranAdvisorIncentive>;
pub type EligibilityThresholds = std::ops::RangeInclusive<usize>;

// TODO: for the sake of clarity, introduce a different naming between ca reviews and vca ranking

// Supposing to have a file with all the rankings for each review
// e.g. something like an expanded version of a AdvisorReviewRow
// [proposal_id, advisor, ratings, ..(other fields from AdvisorReviewRow).., ranking (good/excellent/filtered out), vca]

fn calc_final_ranking_per_review(rankings: &[impl Borrow<VeteranRankingRow>]) -> ReviewRanking {
    let rankings_majority = Decimal::from(rankings.len()) / Decimal::from(2);
    let ranks = rankings.iter().counts_by(|r| r.borrow().score());

    match (ranks.get(&FilteredOut), ranks.get(&Excellent)) {
        (Some(filtered_out), _) if Decimal::from(*filtered_out) >= rankings_majority => {
            ReviewRanking::FilteredOut
        }
        (_, Some(excellent)) if Decimal::from(*excellent) > rankings_majority => {
            ReviewRanking::Excellent
        }
        _ => ReviewRanking::Good,
    }
}

fn rewards_disagreement_discount(agreement_rate: Decimal) -> Decimal {
    // Cannot use decimal range pattern in matches, and don't want to complicate
    // stuff by using exact integer arithmetic since it's not really needed at this point
    // Thresholds at 0.9 - 0.75 - 0.6
    if agreement_rate >= Decimal::new(9, 1) {
        Decimal::ONE
    } else if agreement_rate >= Decimal::new(75, 2) {
        Decimal::from(5) / Decimal::from(6)
    } else if agreement_rate >= Decimal::new(6, 1) {
        Decimal::from(2) / Decimal::from(3)
    } else {
        Decimal::ZERO
    }
}

fn reputation_disagreement_discount(agreement_rate: Decimal) -> Decimal {
    if agreement_rate >= Decimal::new(6, 1) {
        Decimal::ONE
    } else {
        Decimal::ZERO
    }
}

fn calc_final_eligible_rankings(
    all_rankings: &HashMap<VeteranAdvisorId, usize>,
    eligible_rankings: HashMap<VeteranAdvisorId, usize>,
    thresholds: EligibilityThresholds,
    discount_rate: impl Fn(Decimal) -> Decimal,
) -> BTreeMap<VeteranAdvisorId, Rewards> {
    eligible_rankings
        .into_iter()
        .filter_map(|(vca, n_rankings)| {
            if n_rankings < *thresholds.start() {
                return None;
            }

            let to_discount = discount_rate(
                Decimal::from(n_rankings) / Decimal::from(*all_rankings.get(&vca).unwrap()),
            );

            let n_rankings = Rewards::from(n_rankings.min(*thresholds.end())) * to_discount;

            Some((vca, n_rankings))
        })
        .collect()
}

pub fn calculate_veteran_advisors_incentives(
    veteran_rankings: &[VeteranRankingRow],
    total_rewards: Rewards,
    rewards_thresholds: EligibilityThresholds,
    reputation_thresholds: EligibilityThresholds,
) -> HashMap<VeteranAdvisorId, VeteranAdvisorIncentive> {
    let final_rankings_per_review = veteran_rankings
        .iter()
        .into_group_map_by(|ranking| ranking.review_id())
        .into_iter()
        .map(|(review, rankings)| (review, calc_final_ranking_per_review(&rankings)))
        .collect::<BTreeMap<_, _>>();

    let rankings_per_vca = veteran_rankings
        .iter()
        .counts_by(|ranking| ranking.vca.clone());

    let eligible_rankings_per_vca = veteran_rankings
        .iter()
        .filter(|ranking| {
            final_rankings_per_review
                .get(&ranking.review_id())
                .unwrap()
                .is_positive()
                == ranking.score().is_positive()
        })
        .counts_by(|ranking| ranking.vca.clone());

    let reputation_eligible_rankings = calc_final_eligible_rankings(
        &rankings_per_vca,
        eligible_rankings_per_vca.clone(),
        reputation_thresholds,
        reputation_disagreement_discount,
    );

    let rewards_eligible_rankings = calc_final_eligible_rankings(
        &rankings_per_vca,
        eligible_rankings_per_vca,
        rewards_thresholds,
        rewards_disagreement_discount,
    );

    let tot_rewards_eligible_rankings = rewards_eligible_rankings.values().sum::<Rewards>();

    reputation_eligible_rankings
        .into_iter()
        .zip(rewards_eligible_rankings.into_iter())
        .map(|((vca, reputation), (_vca2, reward))| {
            assert_eq!(vca, _vca2); // the use of BTreeMaps ensures iteration is consistent
            (
                vca,
                VeteranAdvisorIncentive {
                    reputation: reputation.to_u64().expect("result does not fit into u64"),
                    rewards: total_rewards * reward / tot_rewards_eligible_rankings,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use std::iter::Iterator;

    const VCA_1: &str = "vca1";
    const VCA_2: &str = "vca2";
    const VCA_3: &str = "vca3";

    struct RandomIterator;
    impl Iterator for RandomIterator {
        type Item = String;
        fn next(&mut self) -> Option<Self::Item> {
            Some(
                (0..10)
                    .map(|_| rand::thread_rng().sample(Alphanumeric) as char)
                    .collect(),
            )
        }
    }

    fn gen_dummy_rankings(
        assessor: String,
        n_excellent: u32,
        n_good: u32,
        n_filtered_out: u32,
        vca: impl Iterator<Item = String>,
    ) -> Vec<VeteranRankingRow> {
        (0..n_excellent)
            .map(|_| ReviewRanking::Excellent)
            .chain((0..n_good).map(|_| ReviewRanking::Good))
            .chain((0..n_filtered_out).map(|_| ReviewRanking::FilteredOut))
            .zip(vca)
            .map(|(ranking, vca)| VeteranRankingRow::dummy(ranking, assessor.clone(), vca))
            .collect()
    }

    #[test]
    fn final_ranking_is_correct() {
        assert!(matches!(
            calc_final_ranking_per_review(&gen_dummy_rankings("".into(), 5, 5, 5, RandomIterator),),
            ReviewRanking::Good
        ));

        assert!(matches!(
            calc_final_ranking_per_review(&gen_dummy_rankings("".into(), 4, 2, 5, RandomIterator)),
            ReviewRanking::Good
        ));

        assert!(matches!(
            calc_final_ranking_per_review(&gen_dummy_rankings("".into(), 4, 1, 5, RandomIterator)),
            ReviewRanking::FilteredOut
        ));

        assert!(matches!(
            calc_final_ranking_per_review(&gen_dummy_rankings("".into(), 3, 1, 1, RandomIterator)),
            ReviewRanking::Excellent
        ));
    }

    #[test]
    fn lower_threshold() {
        let vcas = vec![VCA_1.to_owned(), VCA_2.to_owned()].into_iter();
        let vca2_only = vec![VCA_2.to_owned()].into_iter();
        let total_rewards = Rewards::ONE;
        let rankings = gen_dummy_rankings("1".into(), 1, 1, 0, vcas)
            .into_iter()
            .chain(gen_dummy_rankings("2".into(), 1, 0, 0, vca2_only))
            .collect::<Vec<_>>();
        // only vca with more than 2 reviews get reputation and rewards
        let results = calculate_veteran_advisors_incentives(&rankings, total_rewards, 2..=2, 2..=2);
        assert!(results.get(VCA_1).is_none());
        let res = results.get(VCA_2).unwrap();
        assert_eq!(res.reputation, 2);
        assert_eq!(res.rewards, total_rewards);
    }

    #[test]
    fn upper_threshold() {
        let vcas = vec![VCA_1.to_owned(), VCA_2.to_owned()].into_iter();
        let vca2_only = vec![VCA_2.to_owned()].into_iter();
        let total_rewards = Rewards::ONE;
        let rankings = gen_dummy_rankings("1".into(), 1, 1, 0, vcas)
            .into_iter()
            .chain(gen_dummy_rankings("2".into(), 1, 0, 0, vca2_only))
            .collect::<Vec<_>>();
        let results = calculate_veteran_advisors_incentives(&rankings, total_rewards, 1..=1, 1..=1);
        let res1 = results.get(VCA_1).unwrap();
        assert_eq!(res1.reputation, 1);
        assert_eq!(res1.rewards, Rewards::ONE / Rewards::from(2));
        let res2 = results.get(VCA_2).unwrap();
        assert_eq!(res2.reputation, 1);
        assert_eq!(res2.rewards, Rewards::ONE / Rewards::from(2));
    }

    fn are_close(a: Decimal, b: Decimal) -> bool {
        const DECIMAL_PRECISION: u32 = 10;
        a.round_dp(DECIMAL_PRECISION) == b.round_dp(DECIMAL_PRECISION)
    }

    #[test]
    fn disagreement_discount_rate() {
        let total_rewards = Rewards::ONE;
        let inputs = [
            (Rewards::new(4, 1), Rewards::ZERO, Rewards::ZERO),
            (
                Rewards::new(6, 1),
                Rewards::from(2) / Rewards::from(3),
                Rewards::ONE,
            ),
            (
                Rewards::new(75, 2),
                Rewards::from(5) / Rewards::from(6),
                Rewards::ONE,
            ),
            (Rewards::new(9, 1), Rewards::ONE, Rewards::ONE),
        ];
        for (agreement, reward_discount, reputation_discount) in inputs {
            let rankings = (0..100)
                .flat_map(|i| {
                    let vcas =
                        vec![VCA_1.to_owned(), VCA_2.to_owned(), VCA_3.to_owned()].into_iter();
                    let (good, filtered_out) = if Rewards::from(i) < agreement * Rewards::from(100)
                    {
                        (3, 0)
                    } else {
                        (2, 1)
                    };
                    gen_dummy_rankings(i.to_string(), 0, good, filtered_out, vcas).into_iter()
                })
                .collect::<Vec<_>>();
            let results =
                calculate_veteran_advisors_incentives(&rankings, total_rewards, 1..=200, 1..=200);
            let expected_reward_portion = agreement * Rewards::from(100) * reward_discount;
            dbg!(expected_reward_portion);
            let expected_rewards = total_rewards
                / (Rewards::from(100 * 2) + expected_reward_portion)
                * expected_reward_portion;
            let res = results.get(VCA_3).unwrap();
            assert_eq!(
                res.reputation,
                (Rewards::from(100) * agreement * reputation_discount)
                    .to_u64()
                    .unwrap()
            );
            assert!(are_close(res.rewards, expected_rewards));
        }
    }
}
