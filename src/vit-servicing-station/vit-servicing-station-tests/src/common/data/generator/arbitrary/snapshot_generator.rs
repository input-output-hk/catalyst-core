use crate::common::data::ArbitraryGenerator;
use crate::common::data::ArbitraryValidVotingTemplateGenerator;
use crate::common::data::{Snapshot, ValidVotingTemplateGenerator};
use chrono::{offset::Utc, Duration};
use fake::{faker::chrono::en::DateTimeBetween, Fake};
use std::iter;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData,
    challenges::Challenge,
    funds::Fund,
    proposals::{ChallengeType, Proposal},
    voteplans::Voteplan,
};

use chrono::DateTime;
use vit_servicing_station_lib::db::models::challenges::ChallengeHighlights;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

type UtcDateTime = DateTime<Utc>;

struct FundDateTimes {
    start: UtcDateTime,
    end: UtcDateTime,
    next: UtcDateTime,
    snapshot: UtcDateTime,
    next_snapshot: UtcDateTime,
}

struct VoteplanDateTimes {
    start: UtcDateTime,
    end: UtcDateTime,
    tally: UtcDateTime,
}

#[derive(Clone)]
pub struct ArbitrarySnapshotGenerator {
    id_generator: ArbitraryGenerator,
    template_generator: ArbitraryValidVotingTemplateGenerator,
}

impl Default for ArbitrarySnapshotGenerator {
    fn default() -> Self {
        Self {
            id_generator: ArbitraryGenerator::new(),
            template_generator: ArbitraryValidVotingTemplateGenerator::new(),
        }
    }
}

impl ArbitrarySnapshotGenerator {
    pub fn funds(&mut self) -> Vec<Fund> {
        let size = self.id_generator.random_size();
        iter::from_fn(|| Some(self.gen_single_fund()))
            .take(size)
            .collect()
    }

    fn gen_single_fund(&mut self) -> Fund {
        let id = self.id_generator.id();
        let dates = self.fund_date_times();
        let fund = ValidVotingTemplateGenerator::next_fund(&mut self.template_generator);

        Fund {
            id: id.abs(),
            fund_name: format!("Fund{}", id),
            fund_goal: fund.goal,
            fund_start_time: dates.start.timestamp(),
            voting_power_threshold: fund.threshold.unwrap().into(),
            fund_end_time: dates.end.timestamp(),
            next_fund_start_time: dates.next.timestamp(),
            registration_snapshot_time: dates.snapshot.timestamp(),
            next_registration_snapshot_time: dates.next_snapshot.timestamp(),
            chain_vote_plans: vec![self.voteplan_with_fund_id(id.abs())],
            challenges: self.challenges_with_fund_id(id.abs()),
        }
    }

    fn gen_single_proposal(&mut self, fund: &Fund) -> FullProposalInfo {
        let id = self.id_generator.next_u32() as i32;
        let proposal = ValidVotingTemplateGenerator::next_proposal(&mut self.template_generator);
        let voteplan = fund.chain_vote_plans.first().unwrap();
        let challenge = fund.challenges.first().unwrap();
        let challenge_id = challenge.id;
        let challenge_info = self
            .template_generator
            .proposals_challenge_info(&challenge.challenge_type);
        let proposal = Proposal {
            internal_id: id.abs(),
            proposal_id: id.abs().to_string(),
            proposal_category: self.template_generator.proposal_category(),
            proposal_title: proposal.proposal_title,
            proposal_summary: proposal.proposal_summary,
            proposal_public_key: self.id_generator.hash(),
            proposal_funds: proposal.proposal_funds.parse().unwrap(),
            proposal_url: proposal.proposal_url,
            proposal_impact_score: proposal.proposal_impact_score.parse().unwrap(),
            reviews_count: 0,
            proposal_files_url: proposal.files_url,
            proposer: self.template_generator.proposer(),
            chain_proposal_id: self.id_generator.hash().as_bytes().to_vec(),
            chain_proposal_index: self.id_generator.next_u32() as i64,
            chain_vote_options: proposal.chain_vote_options,
            chain_voteplan_id: fund
                .chain_vote_plans
                .get(0)
                .unwrap()
                .chain_voteplan_id
                .clone(),
            chain_vote_start_time: voteplan.chain_vote_start_time,
            chain_vote_end_time: voteplan.chain_vote_end_time,
            chain_committee_end_time: voteplan.chain_committee_end_time,
            chain_voteplan_payload: voteplan.chain_voteplan_payload.clone(),
            chain_vote_encryption_key: voteplan.chain_vote_encryption_key.clone(),
            fund_id: fund.id,
            challenge_id,
        };

        FullProposalInfo {
            proposal,
            challenge_info,
            challenge_type: challenge.challenge_type.clone(),
        }
    }

    fn fund_date_times(&self) -> FundDateTimes {
        let range_start_time = Utc::now() - Duration::days(10);
        let range_end_time = Utc::now() + Duration::days(10);
        let range_next_start_time = range_end_time + Duration::days(10);
        let start = DateTimeBetween(range_start_time, Utc::now()).fake::<UtcDateTime>();
        let end = DateTimeBetween(Utc::now(), range_end_time).fake::<UtcDateTime>();
        let next = DateTimeBetween(range_end_time, range_next_start_time).fake::<UtcDateTime>();
        let snapshot = DateTimeBetween(start, end).fake::<UtcDateTime>();
        let next_snapshot = DateTimeBetween(end, end + Duration::days(30)).fake::<UtcDateTime>();

        FundDateTimes {
            start,
            end,
            next,
            snapshot,
            next_snapshot,
        }
    }

    fn voteplan_date_times(&self) -> VoteplanDateTimes {
        let range_start_time = Utc::now() - Duration::days(10);
        let range_end_time = Utc::now() + Duration::days(10);
        let range_tally_time = range_end_time + Duration::days(10);
        let start = DateTimeBetween(range_start_time, Utc::now()).fake::<UtcDateTime>();
        let end = DateTimeBetween(Utc::now(), range_end_time).fake::<UtcDateTime>();
        let tally = DateTimeBetween(range_end_time, range_tally_time).fake::<UtcDateTime>();
        VoteplanDateTimes { start, end, tally }
    }

    pub fn voteplans(&mut self, funds: &[Fund]) -> Vec<Voteplan> {
        funds
            .iter()
            .map(|x| self.voteplan_with_fund_id(x.id))
            .collect()
    }

    pub fn challenges(&mut self, funds: &[Fund]) -> Vec<Challenge> {
        funds
            .iter()
            .map(|x| x.challenges.first().unwrap())
            .cloned()
            .collect()
    }

    pub fn token(&mut self) -> (String, ApiTokenData) {
        self.id_generator.token()
    }

    pub fn proposals(&mut self, funds: &[Fund]) -> Vec<FullProposalInfo> {
        funds.iter().map(|x| self.gen_single_proposal(x)).collect()
    }

    pub fn advisor_reviews(&mut self, funds: &[FullProposalInfo]) -> Vec<AdvisorReview> {
        funds
            .iter()
            .map(|x| self.review_with_proposal_id(x.proposal.internal_id))
            .collect()
    }

    pub fn voteplan_with_fund_id(&mut self, fund_id: i32) -> Voteplan {
        let id = self.id_generator.next_u32() as i32;
        let dates = self.voteplan_date_times();

        Voteplan {
            id: id.abs(),
            chain_voteplan_id: self.id_generator.hash(),
            chain_vote_start_time: dates.start.timestamp(),
            chain_vote_end_time: dates.end.timestamp(),
            chain_committee_end_time: dates.tally.timestamp(),
            chain_voteplan_payload: "public".to_string(),
            chain_vote_encryption_key: "".to_string(),
            fund_id,
        }
    }

    pub fn challenges_with_fund_id(&mut self, fund_id: i32) -> Vec<Challenge> {
        let simple_id = self.id_generator.next_u32() as i32;
        let community_choice_id = self.id_generator.next_u32() as i32;

        let first_challenge = self.template_generator.next_challenge();
        let second_challenge = self.template_generator.next_challenge();

        vec![
            Challenge {
                id: simple_id.abs(),
                challenge_type: ChallengeType::Simple,
                title: first_challenge.title,
                description: first_challenge.description,
                rewards_total: first_challenge.rewards_total.parse().unwrap(),
                proposers_rewards: first_challenge.proposers_rewards.parse().unwrap(),
                fund_id,
                challenge_url: self.template_generator.gen_http_address(),
                highlights: None,
            },
            Challenge {
                id: community_choice_id.abs(),
                challenge_type: ChallengeType::CommunityChoice,
                title: second_challenge.title,
                description: second_challenge.description,
                rewards_total: second_challenge.rewards_total.parse().unwrap(),
                proposers_rewards: second_challenge.proposers_rewards.parse().unwrap(),
                fund_id,
                challenge_url: self.template_generator.gen_http_address(),
                highlights: Some(ChallengeHighlights {
                    sponsor: "Foobar".to_string(),
                }),
            },
        ]
    }

    pub fn challenge_with_fund_id(&mut self, fund_id: i32) -> Challenge {
        let id = self.id_generator.next_u32() as i32;
        let challenge = self.template_generator.next_challenge();

        Challenge {
            id: id.abs(),
            challenge_type: ChallengeType::CommunityChoice,
            title: challenge.title,
            description: challenge.description,
            rewards_total: challenge.rewards_total.parse().unwrap(),
            proposers_rewards: challenge.proposers_rewards.parse().unwrap(),
            fund_id,
            challenge_url: self.template_generator.gen_http_address(),
            highlights: None,
        }
    }

    pub fn review_with_proposal_id(&mut self, proposal_id: i32) -> AdvisorReview {
        let id = self.id_generator.next_u32() as i32;
        let review = (self.template_generator).next_review();
        AdvisorReview {
            id,
            proposal_id,
            assessor: review.assessor,
            impact_alignment_rating_given: review.impact_alignment_rating_given,
            impact_alignment_note: review.impact_alignment_note,
            feasibility_rating_given: review.feasibility_rating_given,
            feasibility_note: review.feasibility_note,
            auditability_rating_given: review.auditability_rating_given,
            auditability_note: review.auditability_note,
        }
    }

    pub fn snapshot(&mut self) -> Snapshot {
        let funds = self.funds();
        let voteplans = self.voteplans(&funds);
        let challenges = self.challenges(&funds);
        let proposals = self.proposals(&funds);
        let reviews = self.advisor_reviews(&proposals);
        let tokens = self.id_generator.tokens();

        Snapshot::new(funds, proposals, challenges, tokens, voteplans, reviews)
    }
}
