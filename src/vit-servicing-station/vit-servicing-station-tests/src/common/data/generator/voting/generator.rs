use std::collections::HashMap;

use super::parameters::SingleVotePlanParameters;
use super::ProposalTemplate;
use crate::common::data::generator::{ArbitraryGenerator, Snapshot, ValidVotingTemplateGenerator};
use crate::common::data::ValidVotePlanParameters;
use chain_impl_mockchain::certificate::{ExternalProposalId, VotePlan};
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::db::models::proposals::ProposalVotePlanCommon;
use vit_servicing_station_lib::db::models::{
    challenges::Challenge,
    funds::Fund,
    proposals::{Category, Proposal, Proposer},
    voteplans::Voteplan,
};

pub struct ValidVotePlanGenerator {
    parameters: ValidVotePlanParameters,
}

impl ValidVotePlanGenerator {
    pub fn new(parameters: ValidVotePlanParameters) -> Self {
        Self { parameters }
    }

    fn convert_to_vote_plan(single_vote_plan: &SingleVotePlanParameters) -> VotePlan {
        single_vote_plan.vote_plan().into()
    }

    pub fn build(&mut self, template_generator: &mut dyn ValidVotingTemplateGenerator) -> Snapshot {
        let mut generator = ArbitraryGenerator::new();

        let fund_template = template_generator.next_fund();
        self.parameters.current_fund.info.fund_goal = fund_template.goal;

        let vote_plans: Vec<Voteplan> = self
            .parameters
            .current_fund
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
                    chain_vote_start_time: self.parameters.current_fund.info.dates.voting_start,
                    chain_vote_end_time: self.parameters.current_fund.info.dates.voting_tally_start,
                    chain_committee_end_time: self
                        .parameters
                        .current_fund
                        .info
                        .dates
                        .voting_tally_end,
                    chain_voteplan_payload: payload_type.to_string(),
                    chain_vote_encryption_key: single_vote_plan
                        .vote_encryption_key()
                        .unwrap_or_default(),
                    fund_id: self.parameters.current_fund.info.fund_id,
                    token_identifier: vote_plan.voting_token().to_string(),
                }
            })
            .collect();

        let challenges: Vec<Challenge> = std::iter::from_fn(|| {
            let challenge_data = template_generator.next_challenge();
            Some(Challenge {
                internal_id: challenge_data.internal_id,
                id: challenge_data.id.parse().unwrap(),
                challenge_type: challenge_data.challenge_type,
                title: challenge_data.title,
                description: challenge_data.description,
                rewards_total: challenge_data.rewards_total.parse().unwrap(),
                proposers_rewards: challenge_data.proposers_rewards.parse().unwrap(),
                fund_id: self.parameters.current_fund.info.fund_id,
                challenge_url: challenge_data.challenge_url,
                highlights: challenge_data.highlight,
            })
        })
        .take(self.parameters.current_fund.challenges_count)
        .collect();

        let mut fund = self
            .parameters
            .current_fund
            .to_fund(vote_plans.clone(), challenges);

        let mut proposals = vec![];

        let mut mirrored_templates = HashMap::<ExternalProposalId, ProposalTemplate>::new();

        for (index, vote_plan) in vote_plans.iter().enumerate() {
            let group = fund
                .groups
                .iter()
                .find(|g| g.token_identifier == vote_plan.token_identifier)
                .unwrap();

            for (index, proposal) in self.parameters.current_fund.vote_plans[index]
                .proposals()
                .iter()
                .enumerate()
            {
                let proposal_template = mirrored_templates
                    .entry(proposal.id())
                    .or_insert_with(|| template_generator.next_proposal())
                    .clone();

                let challenge_idx: i32 = proposal_template.challenge_id.unwrap().parse().unwrap();
                let challenge = fund
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

                if self
                    .parameters
                    .current_fund
                    .calculate_challenges_total_funds
                {
                    challenge.rewards_total += proposal_funds;
                }

                let proposal_internal_id = proposal_template.internal_id.parse().unwrap();

                let proposal = Proposal {
                    internal_id: proposal_internal_id,
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
                    chain_vote_options: self.parameters.current_fund.vote_options.clone(),
                    chain_vote_start_time: fund.stage_dates.voting_start,
                    chain_vote_end_time: fund.stage_dates.voting_end,
                    chain_committee_end_time: fund.stage_dates.tallying_end,
                    chain_voteplan_payload: vote_plan.chain_voteplan_payload.clone(),
                    chain_vote_encryption_key: vote_plan.chain_vote_encryption_key.clone(),
                    fund_id: fund.id,
                    challenge_id: challenge.id,
                    extra: Some(
                        vec![("key1", "value1"), ("key2", "value2")]
                            .into_iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect(),
                    ),
                };

                proposals.push(FullProposalInfo {
                    proposal,
                    challenge_info: proposal_template.proposal_challenge_info,
                    challenge_type: challenge.challenge_type.clone(),
                    voteplan: ProposalVotePlanCommon {
                        chain_voteplan_id: vote_plan.chain_voteplan_id.clone(),
                        chain_proposal_index: index as i64,
                    },
                    group_id: group.group_id.clone(),
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
        .take(self.parameters.current_fund.reviews_count)
        .collect();

        let goals = fund.goals.clone();

        let mut funds = vec![fund];
        let next_funds: Vec<Fund> = self
            .parameters
            .next_funds
            .iter()
            .cloned()
            .map(Into::into)
            .collect();
        funds.extend(next_funds);

        Snapshot::new(
            funds,
            proposals,
            challenges,
            generator.tokens(),
            vote_plans,
            reviews,
            goals,
        )
    }
}
