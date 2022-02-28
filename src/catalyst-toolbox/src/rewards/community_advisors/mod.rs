mod funding;
mod lottery;

use crate::community_advisors::models::{AdvisorReviewRow, ReviewRanking};
use lottery::{CasWinnings, TicketsDistribution};
use rand::{Rng, SeedableRng};
use rand_chacha::{ChaCha8Rng, ChaChaRng};

use std::collections::{BTreeMap, BTreeSet};

pub use crate::rewards::{community_advisors::funding::ProposalRewardSlots, Funds, Rewards};
pub use funding::FundSetting;

pub type Seed = <ChaChaRng as SeedableRng>::Seed;
pub type CommunityAdvisor = String;
pub type ProposalId = String;

pub type CaRewards = BTreeMap<CommunityAdvisor, Rewards>;
pub type ProposalsReviews = BTreeMap<ProposalId, Vec<AdvisorReviewRow>>;
pub type ApprovedProposals = BTreeMap<ProposalId, Funds>;

const LEGACY_MAX_WINNING_TICKETS: u64 = 3;

#[derive(Debug)]
struct ProposalRewards {
    per_ticket_reward: Rewards,
    tickets: ProposalTickets,
}

#[derive(Debug)]
enum ProposalTickets {
    Legacy {
        eligible_assessors: BTreeSet<CommunityAdvisor>,
        winning_tkts: u64,
    },
    Fund7 {
        excellent_tkts: TicketsDistribution,
        good_tkts: TicketsDistribution,
        excellent_winning_tkts: u64,
        good_winning_tkts: u64,
    },
}

fn get_tickets_per_proposal(
    proposal_reviews: ProposalsReviews,
    rewards_slots: &ProposalRewardSlots,
) -> (u64, BTreeMap<ProposalId, ProposalTickets>) {
    let (winning_tickets, proposals_tickets): (Vec<_>, _) = proposal_reviews
        .into_iter()
        .map(|(id, reviews)| {
            let filtered = reviews
                .into_iter()
                .filter(|review| !matches!(review.score(), ReviewRanking::FilteredOut))
                .collect::<Vec<_>>();
            let tickets = load_tickets_from_reviews(&filtered, rewards_slots);
            let winning_tickets = match tickets {
                ProposalTickets::Legacy { winning_tkts, .. } => {
                    // it would be a bit harder to track it otherwise, and we don't need this additional
                    // complexity now
                    assert_eq!(
                        0,
                        rewards_slots.max_winning_tickets() % LEGACY_MAX_WINNING_TICKETS
                    );
                    winning_tkts
                        * (rewards_slots.max_winning_tickets() / LEGACY_MAX_WINNING_TICKETS)
                }
                ProposalTickets::Fund7 {
                    excellent_winning_tkts,
                    good_winning_tkts,
                    ..
                } => excellent_winning_tkts + good_winning_tkts,
            };

            (winning_tickets, (id, tickets))
        })
        .unzip();

    (winning_tickets.into_iter().sum(), proposals_tickets)
}

fn calculate_rewards_per_proposal(
    proposal_reviews: ProposalsReviews,
    approved_proposals: &ApprovedProposals,
    funding: &FundSetting,
    rewards_slots: &ProposalRewardSlots,
) -> Vec<ProposalRewards> {
    let bonus_funds = funding.bonus_funds();

    let total_approved_budget = approved_proposals.values().sum::<Funds>();
    let (total_tickets, proposals_tickets) =
        get_tickets_per_proposal(proposal_reviews, rewards_slots);

    let base_ticket_reward = funding.proposal_funds() / Rewards::from(total_tickets);

    proposals_tickets
        .into_iter()
        .map(|(id, tickets)| {
            let bonus_reward = approved_proposals
                .get(&id)
                .map(|budget| bonus_funds * budget / total_approved_budget)
                .unwrap_or_default();
            let per_ticket_reward = match tickets {
                ProposalTickets::Legacy { winning_tkts, .. } => {
                    base_ticket_reward * Rewards::from(rewards_slots.max_winning_tickets())
                        / Rewards::from(LEGACY_MAX_WINNING_TICKETS)
                        + bonus_reward / Rewards::from(winning_tkts)
                }
                ProposalTickets::Fund7 {
                    excellent_winning_tkts,
                    good_winning_tkts,
                    ..
                } => {
                    base_ticket_reward
                        + bonus_reward / Rewards::from(excellent_winning_tkts + good_winning_tkts)
                }
            };
            ProposalRewards {
                tickets,
                per_ticket_reward,
            }
        })
        .collect()
}

fn load_tickets_from_reviews(
    proposal_reviews: &[AdvisorReviewRow],
    rewards_slots: &ProposalRewardSlots,
) -> ProposalTickets {
    let is_legacy = proposal_reviews
        .iter()
        .any(|rev| matches!(rev.score(), ReviewRanking::NA));

    if is_legacy {
        return ProposalTickets::Legacy {
            eligible_assessors: proposal_reviews
                .iter()
                .map(|rev| rev.assessor.clone())
                .collect(),
            winning_tkts: std::cmp::min(proposal_reviews.len() as u64, LEGACY_MAX_WINNING_TICKETS),
        };
    }

    // assuming only one review per assessor in a single proposal
    let (excellent_tkts, good_tkts): (TicketsDistribution, TicketsDistribution) =
        // a full match is used so that we don't forget to consider new review types which may be added in the future
        proposal_reviews.iter().map(|rev| match rev.score() {
            ReviewRanking::Excellent => (rev.assessor.clone(), rewards_slots.excellent_slots),
            ReviewRanking::Good => (rev.assessor.clone(), rewards_slots.good_slots),
            _ => unreachable!("we've already filtered out other review scores"),
        }).partition(|(_ca, tkts)| *tkts == rewards_slots.excellent_slots);

    let excellent_winning_tkts = std::cmp::min(
        excellent_tkts.len() as u64,
        rewards_slots.max_excellent_reviews,
    ) * rewards_slots.excellent_slots;
    let good_winning_tkts = std::cmp::min(good_tkts.len() as u64, rewards_slots.max_good_reviews)
        * rewards_slots.good_slots;

    ProposalTickets::Fund7 {
        excellent_winning_tkts,
        good_winning_tkts,
        excellent_tkts,
        good_tkts,
    }
}

// Run a two stage lottery to reward community advisors
//
// In the first round, only excellent reviews will be taken into consideration
// In the second, losing tickets from the first round will compete with good review
fn double_lottery<R: Rng>(
    stage1: TicketsDistribution,
    mut stage2: TicketsDistribution,
    distribute_first_round: u64,
    distribute_second_round: u64,
    rng: &mut R,
) -> CasWinnings {
    let (mut stage1_winners, stage1_losers) =
        lottery::lottery_distribution(stage1, distribute_first_round, rng);
    stage2.extend(stage1_losers);
    let (stage2_winners, _stage2_losers) =
        lottery::lottery_distribution(stage2, distribute_second_round, rng);
    for (ca, winnings) in stage2_winners {
        *stage1_winners.entry(ca).or_default() += winnings;
    }
    assert_eq!(
        stage1_winners.values().sum::<u64>(),
        distribute_second_round + distribute_first_round
    );
    stage1_winners
}

fn calculate_ca_rewards_for_proposal<R: Rng>(
    proposal_reward: ProposalRewards,
    rng: &mut R,
) -> CaRewards {
    let ProposalRewards {
        tickets,
        per_ticket_reward,
    } = proposal_reward;

    let rewards = match tickets {
        ProposalTickets::Fund7 {
            excellent_winning_tkts,
            good_winning_tkts,
            excellent_tkts,
            good_tkts,
        } => double_lottery(
            excellent_tkts,
            good_tkts,
            excellent_winning_tkts,
            good_winning_tkts,
            rng,
        ),
        ProposalTickets::Legacy {
            eligible_assessors,
            winning_tkts,
        } => {
            lottery::lottery_distribution(
                eligible_assessors.into_iter().map(|ca| (ca, 1)).collect(),
                winning_tkts,
                rng,
            )
            .0
        }
    };

    rewards
        .into_iter()
        .map(|(ca, tickets_won)| (ca, Rewards::from(tickets_won) * per_ticket_reward))
        .collect()
}

pub fn calculate_ca_rewards(
    proposal_reviews: ProposalsReviews,
    approved_proposals: &ApprovedProposals,
    funding: &FundSetting,
    rewards_slots: &ProposalRewardSlots,
    seed: Seed,
) -> CaRewards {
    let proposal_rewards = calculate_rewards_per_proposal(
        proposal_reviews,
        approved_proposals,
        funding,
        rewards_slots,
    );
    let mut ca_rewards = CaRewards::new();
    let mut rng = ChaCha8Rng::from_seed(seed);

    for proposal_reward in proposal_rewards {
        let rewards = calculate_ca_rewards_for_proposal(proposal_reward, &mut rng);

        for (ca, rewards) in rewards {
            *ca_rewards.entry(ca).or_insert(Rewards::ZERO) += rewards;
        }
    }

    ca_rewards
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_dummy_reviews(n_excellent: u32, n_good: u32, n_na: u32) -> Vec<AdvisorReviewRow> {
        (0..n_excellent)
            .map(|_| AdvisorReviewRow::dummy(ReviewRanking::Excellent))
            .chain((0..n_good).map(|_| AdvisorReviewRow::dummy(ReviewRanking::Good)))
            .chain((0..n_na).map(|_| AdvisorReviewRow::dummy(ReviewRanking::NA)))
            .collect()
    }

    #[test]
    fn test_legacy_mode() {
        let reviews = gen_dummy_reviews(5, 10, 1);
        assert!(matches!(
            load_tickets_from_reviews(&reviews, &ProposalRewardSlots::default()),
            ProposalTickets::Legacy {
                winning_tkts: LEGACY_MAX_WINNING_TICKETS,
                ..
            }
        ));
    }

    macro_rules! check_fund6_winning_tkts {
        ($excellent:expr, $good:expr, $expected:expr) => {
            let p = gen_dummy_reviews($excellent, $good, 0);
            match load_tickets_from_reviews(&p, &Default::default()) {
                ProposalTickets::Fund7 {
                    excellent_winning_tkts,
                    good_winning_tkts,
                    ..
                } => assert_eq!(excellent_winning_tkts + good_winning_tkts, $expected),
                _ => panic!("invalid lottery setup"),
            }
        };
    }

    #[test]
    fn test_reviews_limits() {
        // testcases taken from presentation slides
        check_fund6_winning_tkts!(3, 2, 32);
        check_fund6_winning_tkts!(5, 5, 36);
        check_fund6_winning_tkts!(1, 3, 24);
        check_fund6_winning_tkts!(5, 0, 24);
        check_fund6_winning_tkts!(0, 3, 12);
    }

    fn are_close(a: Funds, b: Funds) -> bool {
        const DECIMAL_PRECISION: u32 = 10;
        a.round_dp(DECIMAL_PRECISION) == b.round_dp(DECIMAL_PRECISION)
    }

    #[test]
    fn test_underbudget_redistribution() {
        let mut proposals = BTreeMap::new();
        proposals.insert("1".into(), gen_dummy_reviews(1, 5, 0)); // winning tickets: 24
        proposals.insert("2".into(), gen_dummy_reviews(2, 3, 0)); // winning tickets: 32
        let res = calculate_ca_rewards(
            proposals,
            &ApprovedProposals::new(),
            &FundSetting {
                proposal_ratio: 100,
                bonus_ratio: 0,
                total: Funds::from(100),
            },
            &Default::default(),
            [0; 32],
        );
        assert!(are_close(res.values().sum::<Funds>(), Funds::from(100)));
    }

    #[test]
    fn test_bonus_distribution() {
        let mut proposals = BTreeMap::new();
        proposals.insert("1".into(), gen_dummy_reviews(1, 5, 0)); // winning tickets: 24
        proposals.insert("2".into(), gen_dummy_reviews(1, 1, 0)); // winning tickets: 16
        proposals.insert("3".into(), gen_dummy_reviews(2, 3, 0)); // winning tickets: 32
        let res = calculate_ca_rewards(
            proposals,
            &vec![("1".into(), Funds::from(2)), ("2".into(), Funds::from(1))]
                .into_iter()
                .collect(),
            &FundSetting {
                proposal_ratio: 80,
                bonus_ratio: 20,
                total: Funds::from(100),
            },
            &Default::default(),
            [0; 32],
        );
        assert!(are_close(res.values().sum::<Funds>(), Funds::from(100)));
    }

    #[test]
    fn test_all() {
        use rand::RngCore;

        let mut proposals = BTreeMap::new();
        let mut approved_proposals = ApprovedProposals::new();
        let mut rng = ChaChaRng::from_seed([0; 32]);
        for i in 0..100 {
            proposals.insert(
                i.to_string(),
                gen_dummy_reviews(rng.next_u32() % 10, rng.next_u32() % 10, rng.next_u32() % 2),
            );
            if rng.gen_bool(0.5) {
                approved_proposals.insert(i.to_string(), Funds::from(rng.next_u32() % 1000));
            }
        }
        let res = calculate_ca_rewards(
            proposals,
            &approved_proposals,
            &FundSetting {
                proposal_ratio: 80,
                bonus_ratio: 20,
                total: Funds::from(100),
            },
            &Default::default(),
            [0; 32],
        );
        assert!(are_close(res.values().sum::<Funds>(), Funds::from(100)));
    }

    #[test]
    fn test_double_stage_lottery() {
        let mut proposals = BTreeMap::new();
        let reviews = gen_dummy_reviews(1, 500, 0); // winning tickets: 24
        let excellent_assessor = reviews[0].assessor.clone();
        proposals.insert("1".into(), reviews);
        let res = calculate_ca_rewards(
            proposals,
            &vec![("1".into(), Funds::from(2))].into_iter().collect(),
            &FundSetting {
                proposal_ratio: 80,
                bonus_ratio: 20,
                total: Funds::from(240),
            },
            &Default::default(),
            [0; 32],
        );
        assert!(are_close(res.values().sum::<Funds>(), Funds::from(240)));
        assert!(are_close(
            *res.get(&excellent_assessor).unwrap(),
            Funds::from(120)
        ));
    }
}
