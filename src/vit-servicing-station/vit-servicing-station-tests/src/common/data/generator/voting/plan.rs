use crate::common::data::generator::{ArbitraryGenerator, Snapshot, ValidVotingTemplateGenerator};
use chain_impl_mockchain::certificate::VotePlan;
use chain_impl_mockchain::testing::scenario::template::ProposalDef;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::funds::FundStageDates;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::db::models::{
    challenges::Challenge,
    funds::Fund,
    proposals::{Category, Proposal, Proposer},
    vote_options::VoteOptions,
    voteplans::Voteplan,
};

pub struct SingleVotePlanParameters {
    vote_plan: VotePlanDef,
    vote_encryption_key: Option<String>,
}

impl SingleVotePlanParameters {
    pub fn proposals(&self) -> Vec<ProposalDef> {
        self.vote_plan.proposals()
    }

    pub fn alias(&self) -> String {
        self.vote_plan.alias()
    }
}

impl From<VotePlanDef> for SingleVotePlanParameters {
    fn from(vote_plan: VotePlanDef) -> Self {
        Self {
            vote_plan,
            vote_encryption_key: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidVotePlanDates {
    pub voting_start: i64,
    pub voting_tally_start: i64,
    pub voting_tally_end: i64,
    pub next_fund_start_time: i64,
    pub registration_snapshot_time: i64,
    pub next_registration_snapshot_time: i64,
    pub insight_sharing_start: i64,
    pub proposal_submission_start: i64,
    pub refine_proposals_start: i64,
    pub finalize_proposals_start: i64,
    pub proposal_assessment_start: i64,
    pub assessment_qa_start: i64,
}

pub struct ValidVotePlanParameters {
    pub fund_name: String,
    pub vote_plans: Vec<SingleVotePlanParameters>,
    pub dates: ValidVotePlanDates,
    pub voting_power_threshold: Option<i64>,
    pub vote_options: Option<VoteOptions>,
    pub challenges_count: usize,
    pub reviews_count: usize,
    pub fund_id: Option<i32>,
    pub calculate_challenges_total_funds: bool,
}

impl ValidVotePlanParameters {
    pub fn from_single(vote_plan: VotePlanDef, dates: ValidVotePlanDates) -> Self {
        let alias = vote_plan.alias();
        Self::new(vec![vote_plan], alias, dates)
    }

    pub fn new(vote_plans: Vec<VotePlanDef>, fund_name: String, dates: ValidVotePlanDates) -> Self {
        Self {
            vote_plans: vote_plans.into_iter().map(Into::into).collect(),
            fund_name,
            voting_power_threshold: Some(8000),
            dates,
            vote_options: Some(VoteOptions::parse_coma_separated_value("blank,yes,no")),
            challenges_count: 4,
            reviews_count: 1,
            fund_id: Some(1),
            calculate_challenges_total_funds: false,
        }
    }

    pub fn set_voting_power_threshold(&mut self, voting_power_threshold: i64) {
        self.voting_power_threshold = Some(voting_power_threshold);
    }

    pub fn set_vote_encryption_key(&mut self, vote_encryption_key: String, alias: &str) {
        let vote_plan = self
            .vote_plans
            .iter_mut()
            .find(|x| x.alias() == alias)
            .unwrap();
        vote_plan.vote_encryption_key = Some(vote_encryption_key);
    }

    pub fn set_voting_dates(&mut self, dates: ValidVotePlanDates) {
        self.dates = dates;
    }

    pub fn set_challenges_count(&mut self, challenges_count: usize) {
        self.challenges_count = challenges_count;
    }

    pub fn set_reviews_count(&mut self, reviews_count: usize) {
        self.reviews_count = reviews_count;
    }

    pub fn set_vote_options(&mut self, vote_options: VoteOptions) {
        self.vote_options = Some(vote_options);
    }

    pub fn set_fund_id(&mut self, fund_id: i32) {
        self.fund_id = Some(fund_id);
    }

    pub fn set_calculate_challenges_total_funds(&mut self, calculate_challenges_total_funds: bool) {
        self.calculate_challenges_total_funds = calculate_challenges_total_funds;
    }
}

pub struct ValidVotePlanGenerator {
    parameters: ValidVotePlanParameters,
}

impl ValidVotePlanGenerator {
    pub fn new(parameters: ValidVotePlanParameters) -> Self {
        Self { parameters }
    }

    fn convert_to_vote_plan(single_vote_plan: &SingleVotePlanParameters) -> VotePlan {
        single_vote_plan.vote_plan.clone().into()
    }

    pub fn build(&mut self, template_generator: &mut dyn ValidVotingTemplateGenerator) -> Snapshot {
        let mut generator = ArbitraryGenerator::new();

        let threshold = self.parameters.voting_power_threshold.unwrap();
        let dates = &self.parameters.dates;
        let fund_template = template_generator.next_fund();
        let fund_id = self.parameters.fund_id.unwrap_or(fund_template.id);

        let vote_plans: Vec<Voteplan> = self
            .parameters
            .vote_plans
            .iter()
            .map(|single_vote_plan| {
                let vote_plan = Self::convert_to_vote_plan(single_vote_plan);

                let payload_type = match vote_plan.payload_type() {
                    chain_impl_mockchain::vote::PayloadType::Public => "public",
                    chain_impl_mockchain::vote::PayloadType::Private => "private",
                };

                Voteplan {
                    id: generator.id(),
                    chain_voteplan_id: vote_plan.to_id().to_string(),
                    chain_vote_start_time: dates.voting_start,
                    chain_vote_end_time: dates.voting_tally_start,
                    chain_committee_end_time: dates.voting_tally_end,
                    chain_voteplan_payload: payload_type.to_string(),
                    chain_vote_encryption_key: single_vote_plan
                        .vote_encryption_key
                        .clone()
                        .unwrap_or_else(|| "".to_string()),
                    fund_id,
                }
            })
            .collect();

        let count = self.parameters.challenges_count;
        let challenges: Vec<Challenge> = std::iter::from_fn(|| {
            let challenge_data = template_generator.next_challenge();
            Some(Challenge {
                id: challenge_data.id.parse().unwrap(),
                challenge_type: challenge_data.challenge_type,
                title: challenge_data.title,
                description: challenge_data.description,
                rewards_total: challenge_data.rewards_total.parse().unwrap(),
                proposers_rewards: challenge_data.proposers_rewards.parse().unwrap(),
                fund_id,
                challenge_url: challenge_data.challenge_url,
                highlights: challenge_data.highlight,
            })
        })
        .take(count)
        .collect();

        let mut fund = Fund {
            id: fund_id,
            fund_name: self.parameters.fund_name.clone(),
            fund_goal: fund_template.goal,
            voting_power_threshold: threshold,
            fund_start_time: dates.voting_start,
            fund_end_time: dates.voting_tally_start,
            next_fund_start_time: dates.next_fund_start_time,
            registration_snapshot_time: dates.registration_snapshot_time,
            next_registration_snapshot_time: dates.next_registration_snapshot_time,
            chain_vote_plans: vote_plans.clone(),
            challenges,
            stage_dates: FundStageDates {
                insight_sharing_start: dates.insight_sharing_start,
                proposal_submission_start: dates.proposal_submission_start,
                refine_proposals_start: dates.refine_proposals_start,
                finalize_proposals_start: dates.finalize_proposals_start,
                proposal_assessment_start: dates.proposal_assessment_start,
                assessment_qa_start: dates.assessment_qa_start,
                snapshot_start: dates.registration_snapshot_time,
                voting_start: dates.voting_start,
                voting_end: dates.voting_tally_start,
                tallying_end: dates.voting_tally_end,
            },
        };

        let mut proposals = vec![];

        for (index, vote_plan) in vote_plans.iter().enumerate() {
            for (index, proposal) in self.parameters.vote_plans[index]
                .proposals()
                .iter()
                .enumerate()
            {
                let proposal_template = template_generator.next_proposal();
                let challenge_idx: i32 = proposal_template.challenge_id.unwrap().parse().unwrap();
                let mut challenge = fund
                    .challenges
                    .iter_mut()
                    .find(|x| x.id == challenge_idx)
                    .unwrap_or_else(|| {
                        panic!(
                            "Cannot find challenge with id: {}. Please set more challenges",
                            challenge_idx
                        )
                    });
                let proposal_funds = proposal_template.proposal_funds.parse().unwrap();
                let chain_vote_options = proposal_template.chain_vote_options.clone();

                if self.parameters.calculate_challenges_total_funds {
                    challenge.rewards_total += proposal_funds;
                }

                let proposal = Proposal {
                    internal_id: proposal_template.internal_id.parse().unwrap(),
                    proposal_id: proposal_template.proposal_id.to_string(),
                    proposal_category: Category {
                        category_id: "".to_string(),
                        category_name: proposal_template.category_name,
                        category_description: "".to_string(),
                    },
                    proposal_title: proposal_template.proposal_title,
                    proposal_summary: proposal_template.proposal_summary,
                    proposal_public_key: generator.hash(),
                    proposal_funds,
                    proposal_url: proposal_template.proposal_url.clone(),
                    proposal_impact_score: proposal_template.proposal_impact_score.parse().unwrap(),
                    reviews_count: 0,
                    proposal_files_url: proposal_template.files_url,
                    proposer: Proposer {
                        proposer_name: proposal_template.proposer_name,
                        proposer_email: "".to_string(),
                        proposer_url: proposal_template.proposer_url,
                        proposer_relevant_experience: proposal_template
                            .proposer_relevant_experience,
                    },
                    chain_proposal_id: proposal.id().to_string().as_bytes().to_vec(),
                    chain_proposal_index: index as i64,
                    chain_vote_options: self
                        .parameters
                        .vote_options
                        .clone()
                        .unwrap_or(chain_vote_options),
                    chain_voteplan_id: vote_plan.chain_voteplan_id.clone(),
                    chain_vote_start_time: vote_plan.chain_vote_start_time,
                    chain_vote_end_time: vote_plan.chain_vote_end_time,
                    chain_committee_end_time: vote_plan.chain_committee_end_time,
                    chain_voteplan_payload: vote_plan.chain_voteplan_payload.clone(),
                    chain_vote_encryption_key: vote_plan.chain_vote_encryption_key.clone(),
                    fund_id: fund.id,
                    challenge_id: challenge.id,
                };

                proposals.push(FullProposalInfo {
                    proposal,
                    challenge_info: proposal_template.proposal_challenge_info,
                    challenge_type: challenge.challenge_type.clone(),
                });
            }
        }
        let challenges = fund.challenges.clone();

        let reviews: Vec<AdvisorReview> = std::iter::from_fn(|| {
            let review_data = template_generator.next_review();

            Some(AdvisorReview {
                id: review_data
                    .id
                    .unwrap_or_else(|| 0i32.to_string())
                    .parse()
                    .unwrap(),
                proposal_id: review_data.proposal_id.parse().unwrap(),
                assessor: review_data.assessor,
                impact_alignment_rating_given: review_data.impact_alignment_rating_given,
                impact_alignment_note: review_data.impact_alignment_note,
                feasibility_rating_given: review_data.feasibility_rating_given,
                feasibility_note: review_data.feasibility_note,
                auditability_rating_given: review_data.auditability_rating_given,
                auditability_note: review_data.auditability_note,
                ranking: review_data.ranking,
            })
        })
        .take(self.parameters.reviews_count)
        .collect();

        Snapshot::new(
            vec![fund],
            proposals,
            challenges,
            generator.tokens(),
            vote_plans,
            reviews,
        )
    }
}
