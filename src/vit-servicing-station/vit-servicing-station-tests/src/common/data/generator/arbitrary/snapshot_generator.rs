use crate::common::data::ArbitraryGenerator;
use crate::common::data::ArbitraryValidVotingTemplateGenerator;
use crate::common::data::{Snapshot, ValidVotingTemplateGenerator};
use chain_impl_mockchain::certificate::ExternalProposalId;
use itertools::Itertools;
use snapshot_lib::voting_group::DEFAULT_DIRECT_VOTER_GROUP;
use snapshot_lib::voting_group::DEFAULT_REPRESENTATIVE_GROUP;
use std::collections::BTreeSet;
use std::iter;
use time::{Duration, OffsetDateTime};
use vit_servicing_station_lib::db::models::funds::FundStageDates;
use vit_servicing_station_lib::db::models::goals::Goal;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::models::proposals::ProposalVotePlanCommon;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData,
    challenges::Challenge,
    funds::Fund,
    proposals::{ChallengeType, Proposal, ProposalChallengeInfo},
    voteplans::Voteplan,
};

use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

pub struct FundDateTimes {
    start: OffsetDateTime,
    end: OffsetDateTime,
    next: OffsetDateTime,
    snapshot: OffsetDateTime,
    next_snapshot: OffsetDateTime,
    insight_sharing_start: OffsetDateTime,
    proposal_submission_start: OffsetDateTime,
    refine_proposals_start: OffsetDateTime,
    finalize_proposals_start: OffsetDateTime,
    proposal_assessment_start: OffsetDateTime,
    assessment_qa_start: OffsetDateTime,
    snapshot_start: OffsetDateTime,
    voting_start: OffsetDateTime,
    voting_end: OffsetDateTime,
    tallying_end: OffsetDateTime,
}

#[derive(Clone)]
pub struct ArbitrarySnapshotGenerator {
    id_generator: ArbitraryGenerator,
    template_generator: ArbitraryValidVotingTemplateGenerator,
    current_fund_id: i32,
    current_proposal_id: i32,
}

impl Default for ArbitrarySnapshotGenerator {
    fn default() -> Self {
        Self {
            id_generator: ArbitraryGenerator::new(),
            template_generator: ArbitraryValidVotingTemplateGenerator::new(),
            current_fund_id: 1,
            current_proposal_id: 1,
        }
    }
}

impl ArbitrarySnapshotGenerator {
    pub fn funds(&mut self) -> Vec<Fund> {
        let size = self.id_generator.random_size();
        let mut f = iter::from_fn(|| Some(self.gen_single_fund()))
            .take(size)
            .collect::<Vec<_>>();

        for i in 1..f.len() {
            f[i - 1].next_fund_start_time = f[i].fund_start_time;
            f[i - 1].next_registration_snapshot_time = f[i].registration_snapshot_time;
        }

        f
    }

    fn gen_single_fund(&mut self) -> Fund {
        let id = self.current_fund_id;
        self.current_fund_id += 1;

        let dates = self.fund_date_times();
        let fund = ValidVotingTemplateGenerator::next_fund(&mut self.template_generator);

        let groups: BTreeSet<Group> = std::iter::from_fn(|| Some(self.id_generator.next_i32()))
            .take(2)
            .map(|group_id| {
                let group_id = if group_id % 2 == 0 {
                    DEFAULT_DIRECT_VOTER_GROUP.to_string()
                } else {
                    DEFAULT_REPRESENTATIVE_GROUP.to_string()
                };

                Group {
                    fund_id: id,
                    token_identifier: format!("group{group_id}-token"),
                    group_id,
                }
            })
            .collect();

        let chain_vote_plans = groups
            .iter()
            .map(|g| self.voteplan_with_fund_id(id, &dates, g.token_identifier.clone()))
            .collect();

        Fund {
            id,
            fund_name: format!("Fund{}", id),
            fund_goal: fund.goal,
            fund_start_time: dates.start.unix_timestamp(),
            voting_power_threshold: fund.threshold.unwrap().into(),
            fund_end_time: dates.end.unix_timestamp(),
            next_fund_start_time: dates.next.unix_timestamp(),
            registration_snapshot_time: dates.snapshot.unix_timestamp(),
            next_registration_snapshot_time: dates.next_snapshot.unix_timestamp(),
            chain_vote_plans,
            challenges: self.challenges_with_fund_id(id),
            stage_dates: FundStageDates {
                insight_sharing_start: dates.insight_sharing_start.unix_timestamp(),
                proposal_submission_start: dates.proposal_submission_start.unix_timestamp(),
                refine_proposals_start: dates.refine_proposals_start.unix_timestamp(),
                finalize_proposals_start: dates.finalize_proposals_start.unix_timestamp(),
                proposal_assessment_start: dates.proposal_assessment_start.unix_timestamp(),
                assessment_qa_start: dates.assessment_qa_start.unix_timestamp(),
                snapshot_start: dates.snapshot_start.unix_timestamp(),
                voting_start: dates.voting_start.unix_timestamp(),
                voting_end: dates.voting_end.unix_timestamp(),
                tallying_end: dates.tallying_end.unix_timestamp(),
            },
            goals: vec![Goal {
                id: 1,
                goal_name: "goal1".into(),
                fund_id: id,
            }],
            results_url: format!("http://localhost/fund/{id}/results/"),
            survey_url: format!("http://localhost/fund/{id}/survey/"),
            groups,
        }
    }

    fn gen_single_proposal(&mut self, fund: &Fund) -> FullProposalInfo {
        let id = self.current_proposal_id;
        self.current_proposal_id += 1;

        let proposal = ValidVotingTemplateGenerator::next_proposal(&mut self.template_generator);
        let voteplan = fund.chain_vote_plans.first().unwrap();
        let challenge = fund.challenges.first().unwrap();
        let challenge_id = challenge.id;
        let challenge_info = self
            .template_generator
            .proposals_challenge_info(&challenge.challenge_type);
        let chain_proposal_id = ExternalProposalId::from(self.id_generator.bytes())
            .to_string()
            .as_bytes()
            .to_vec();

        let extra = match &challenge_info {
            ProposalChallengeInfo::Simple(info) => {
                vec![("solution", info.proposal_solution.as_str())]
            }
            ProposalChallengeInfo::CommunityChoice(info) => vec![
                ("brief", info.proposal_brief.as_str()),
                ("importance", info.proposal_importance.as_str()),
                ("goal", info.proposal_goal.as_str()),
                ("metrics", info.proposal_metrics.as_str()),
            ],
        }
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect::<std::collections::BTreeMap<String, String>>();

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
            chain_proposal_id,
            chain_vote_options: proposal.chain_vote_options,
            chain_vote_start_time: voteplan.chain_vote_start_time,
            chain_vote_end_time: voteplan.chain_vote_end_time,
            chain_committee_end_time: voteplan.chain_committee_end_time,
            chain_voteplan_payload: voteplan.chain_voteplan_payload.clone(),
            chain_vote_encryption_key: voteplan.chain_vote_encryption_key.clone(),
            fund_id: fund.id,
            extra: Some(extra),
            challenge_id,
        };

        FullProposalInfo {
            proposal,
            challenge_info,
            challenge_type: challenge.challenge_type.clone(),
            voteplan: ProposalVotePlanCommon {
                chain_proposal_index: self.id_generator.next_u32() as i64,
                chain_voteplan_id: fund
                    .chain_vote_plans.first()
                    .unwrap()
                    .chain_voteplan_id
                    .clone(),
            },
            group_id: fund.groups.iter().next().unwrap().group_id.clone(),
        }
    }

    fn fund_date_times(&self) -> FundDateTimes {
        let range_start_time = OffsetDateTime::now_utc() - Duration::days(10);
        let range_end_time = OffsetDateTime::now_utc() + Duration::days(10);
        let range_next_start_time = range_end_time + Duration::days(10);
        let start = rand_datetime_in_range(range_start_time, OffsetDateTime::now_utc());
        let end = rand_datetime_in_range(OffsetDateTime::now_utc(), range_end_time);
        let next = rand_datetime_in_range(range_end_time, range_next_start_time);
        let snapshot = rand_datetime_in_range(start, end);
        let next_snapshot = rand_datetime_in_range(end, end + Duration::days(30));

        let insight_sharing_start = rand_datetime_in_range(start, end);
        let proposal_submission_start = rand_datetime_in_range(insight_sharing_start, end);
        let refine_proposals_start = rand_datetime_in_range(proposal_submission_start, end);
        let finalize_proposals_start = rand_datetime_in_range(refine_proposals_start, end);
        let proposal_assessment_start = rand_datetime_in_range(finalize_proposals_start, end);
        let assessment_qa_start = rand_datetime_in_range(finalize_proposals_start, end);
        let snapshot_start = rand_datetime_in_range(assessment_qa_start, end);
        let voting_start = rand_datetime_in_range(snapshot_start, end);
        let voting_end = rand_datetime_in_range(voting_start, end);
        let tallying_end = rand_datetime_in_range(voting_end, end);

        FundDateTimes {
            start,
            end,
            next,
            snapshot,
            next_snapshot,
            insight_sharing_start,
            proposal_submission_start,
            refine_proposals_start,
            finalize_proposals_start,
            proposal_assessment_start,
            assessment_qa_start,
            snapshot_start,
            voting_start,
            voting_end,
            tallying_end,
        }
    }

    pub fn voteplans(&mut self, funds: &[Fund]) -> Vec<Voteplan> {
        funds
            .iter()
            .flat_map(|f| f.chain_vote_plans.iter())
            .cloned()
            .collect()
    }

    pub fn challenges(&mut self, funds: &[Fund]) -> Vec<Challenge> {
        funds
            .iter()
            .flat_map(|x| &x.challenges)
            .sorted_by(|a, b| a.internal_id.cmp(&b.internal_id))
            .cloned()
            .collect()
    }

    pub fn token(&mut self) -> (String, ApiTokenData) {
        self.id_generator.token()
    }

    pub fn proposals(&mut self, funds: &[Fund]) -> Vec<FullProposalInfo> {
        funds.iter().map(|x| self.gen_single_proposal(x)).collect()
    }

    pub fn advisor_reviews(&mut self, proposals: &mut [FullProposalInfo]) -> Vec<AdvisorReview> {
        proposals
            .iter_mut()
            .map(|x| {
                x.proposal.reviews_count += 1;
                self.review_with_proposal_id(x.proposal.internal_id)
            })
            .collect()
    }

    pub fn goals(&mut self, funds: &[Fund]) -> Vec<Goal> {
        funds
            .iter()
            .enumerate()
            .map(|(i, f)| Goal {
                id: i as i32,
                goal_name: format!("goal{i}"),
                fund_id: f.id,
            })
            .collect()
    }

    pub fn voteplan_with_fund_id(
        &mut self,
        fund_id: i32,
        dates: &FundDateTimes,
        token_identifier: String,
    ) -> Voteplan {
        let id = self.id_generator.next_u32() as i32;

        Voteplan {
            id: id.abs(),
            chain_voteplan_id: self.id_generator.hash(),
            chain_vote_start_time: dates.voting_start.unix_timestamp(),
            chain_vote_end_time: dates.voting_end.unix_timestamp(),
            chain_committee_end_time: dates.tallying_end.unix_timestamp(),
            chain_voteplan_payload: "public".to_string(),
            chain_vote_encryption_key: "".to_string(),
            fund_id,
            token_identifier,
        }
    }

    pub fn challenges_with_fund_id(&mut self, fund_id: i32) -> Vec<Challenge> {
        let simple_id = self.id_generator.next_u32() as i32;
        let community_choice_id = self.id_generator.next_u32() as i32;

        let first_challenge = self.template_generator.next_challenge();
        let second_challenge = self.template_generator.next_challenge();

        let mut challenges = vec![
            Challenge {
                internal_id: first_challenge.internal_id,
                id: simple_id.abs(),
                challenge_type: ChallengeType::Simple,
                title: first_challenge.title,
                description: first_challenge.description,
                rewards_total: first_challenge.rewards_total.parse().unwrap(),
                proposers_rewards: first_challenge.proposers_rewards.parse().unwrap(),
                fund_id,
                challenge_url: self.template_generator.gen_challenge_url(),
                highlights: self.template_generator.gen_highlights(),
            },
            Challenge {
                internal_id: second_challenge.internal_id,
                id: community_choice_id.abs(),
                challenge_type: ChallengeType::CommunityChoice,
                title: second_challenge.title,
                description: second_challenge.description,
                rewards_total: second_challenge.rewards_total.parse().unwrap(),
                proposers_rewards: second_challenge.proposers_rewards.parse().unwrap(),
                fund_id,
                challenge_url: self.template_generator.gen_challenge_url(),
                highlights: self.template_generator.gen_highlights(),
            },
        ];

        challenges.sort_by_key(|c| c.id);

        challenges
    }

    pub fn challenge_with_fund_id(&mut self, fund_id: i32) -> Challenge {
        let id = self.id_generator.next_u32() as i32;
        let challenge = self.template_generator.next_challenge();

        Challenge {
            internal_id: challenge.internal_id,
            id: id.abs(),
            challenge_type: ChallengeType::CommunityChoice,
            title: challenge.title,
            description: challenge.description,
            rewards_total: challenge.rewards_total.parse().unwrap(),
            proposers_rewards: challenge.proposers_rewards.parse().unwrap(),
            fund_id,
            challenge_url: self.template_generator.gen_http_address(),
            highlights: challenge.highlight,
        }
    }

    pub fn goals_with_fund_id(&mut self, fund_id: i32) -> Vec<Goal> {
        let id = (self.id_generator.next_u32() % (i32::MAX as u32)) as i32;

        vec![
            Goal {
                fund_id,
                id,
                goal_name: "goal1".into(),
            },
            Goal {
                fund_id,
                id,
                goal_name: "goal2".into(),
            },
        ]
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
            ranking: review.ranking,
        }
    }

    pub fn snapshot(&mut self) -> Snapshot {
        let funds = self.funds();
        let voteplans = self.voteplans(&funds);
        let challenges = self.challenges(&funds);
        let mut proposals = self.proposals(&funds);
        let reviews = self.advisor_reviews(&mut proposals);
        let goals = self.goals(&funds);
        let tokens = self.id_generator.tokens();

        Snapshot::new(
            funds, proposals, challenges, tokens, voteplans, reviews, goals,
        )
    }
}

fn rand_datetime_in_range(left: OffsetDateTime, right: OffsetDateTime) -> OffsetDateTime {
    use rand::Rng;
    let left_timestamp = left.unix_timestamp();
    let right_timestamp = right.unix_timestamp();
    OffsetDateTime::from_unix_timestamp(
        rand::thread_rng().gen_range(left_timestamp, right_timestamp),
    )
    .unwrap()
}
