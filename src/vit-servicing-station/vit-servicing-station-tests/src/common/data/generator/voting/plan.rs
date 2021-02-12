use crate::common::data::generator::{ArbitraryGenerator, Snapshot, ValidVotingTemplateGenerator};
use chain_impl_mockchain::certificate::VotePlan;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use rand::{rngs::OsRng, RngCore};
use vit_servicing_station_lib::db::models::{
    challenges::Challenge,
    funds::Fund,
    proposals::{Category, Proposal, Proposer},
    vote_options::VoteOptions,
    voteplans::Voteplan,
};

pub struct ValidVotePlanParameters {
    pub vote_plan: VotePlanDef,
    pub voting_power_threshold: Option<i64>,
    pub voting_start: Option<i64>,
    pub voting_tally_start: Option<i64>,
    pub voting_tally_end: Option<i64>,
    pub next_fund_start_time: Option<i64>,
    pub vote_encryption_key: Option<String>,
    pub vote_options: Option<VoteOptions>,
    pub challenges_count: usize,
    pub fund_id: Option<i32>,
    pub calculate_challenges_total_funds: bool,
}

impl ValidVotePlanParameters {
    pub fn new(vote_plan: VotePlanDef) -> Self {
        Self {
            vote_plan,
            voting_power_threshold: Some(8000),
            voting_start: None,
            voting_tally_start: None,
            voting_tally_end: None,
            next_fund_start_time: None,
            vote_encryption_key: None,
            vote_options: Some(VoteOptions::parse_coma_separated_value("blank,yes,no")),
            challenges_count: 4,
            fund_id: Some(1),
            calculate_challenges_total_funds: false,
        }
    }

    pub fn set_voting_power_threshold(&mut self, voting_power_threshold: i64) {
        self.voting_power_threshold = Some(voting_power_threshold);
    }

    pub fn set_vote_encryption_key(&mut self, vote_encryption_key: String) {
        self.vote_encryption_key = Some(vote_encryption_key);
    }

    pub fn set_voting_start(&mut self, voting_start: i64) {
        self.voting_start = Some(voting_start);
    }

    pub fn set_voting_tally_start(&mut self, voting_tally_start: i64) {
        self.voting_tally_start = Some(voting_tally_start);
    }

    pub fn set_voting_tally_end(&mut self, voting_tally_end: i64) {
        self.voting_tally_end = Some(voting_tally_end);
    }

    pub fn set_next_fund_start_time(&mut self, next_fund_start_time: i64) {
        self.next_fund_start_time = Some(next_fund_start_time);
    }

    pub fn set_challenges_count(&mut self, challenges_count: usize) {
        self.challenges_count = challenges_count;
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

    fn convert_to_vote_plan(vote_plan_def: &VotePlanDef) -> VotePlan {
        vote_plan_def.clone().into()
    }

    pub fn build(&mut self, template_generator: &mut dyn ValidVotingTemplateGenerator) -> Snapshot {
        let mut generator = ArbitraryGenerator::new();
        let vote_plan = Self::convert_to_vote_plan(&self.parameters.vote_plan);
        let chain_vote_plan_id = vote_plan.to_id().to_string();
        let threshold = self.parameters.voting_power_threshold.unwrap();
        let voting_start = self.parameters.voting_start.unwrap();
        let voting_tally_start = self.parameters.voting_tally_start.unwrap();
        let voting_tally_end = self.parameters.voting_tally_end.unwrap();
        let next_fund_start_time = self.parameters.next_fund_start_time.unwrap();

        let fund_template = template_generator.next_fund();
        let fund_id = self.parameters.fund_id.unwrap_or(fund_template.id);

        let payload_type = match vote_plan.payload_type() {
            chain_impl_mockchain::vote::PayloadType::Public => "public",
            chain_impl_mockchain::vote::PayloadType::Private => "private",
        };

        let vote_plan = Voteplan {
            id: generator.id(),
            chain_voteplan_id: chain_vote_plan_id.clone(),
            chain_vote_start_time: voting_start,
            chain_vote_end_time: voting_tally_start,
            chain_committee_end_time: voting_tally_end,
            chain_voteplan_payload: payload_type.to_string(),
            chain_vote_encryption_key: self
                .parameters
                .vote_encryption_key
                .clone()
                .unwrap_or_else(|| "".to_string()),
            fund_id,
        };

        let count = self.parameters.challenges_count;
        let challenges = std::iter::from_fn(|| {
            let challenge_data = template_generator.next_challenge();
            Some(Challenge {
                id: challenge_data.id.parse().unwrap(),
                title: challenge_data.title,
                description: challenge_data.description,
                rewards_total: challenge_data.rewards_total.parse().unwrap(),
                fund_id,
                challenge_url: challenge_data.challenge_url,
            })
        })
        .take(count)
        .collect();

        let naive = NaiveDateTime::from_timestamp(voting_start, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        let mut fund = Fund {
            id: fund_id,
            fund_name: self.parameters.vote_plan.alias(),
            fund_goal: fund_template.goal,
            voting_power_info: datetime.to_rfc3339_opts(SecondsFormat::Secs, true),
            voting_power_threshold: threshold,
            rewards_info: fund_template.rewards_info,
            fund_start_time: voting_start,
            fund_end_time: voting_tally_end,
            next_fund_start_time,
            chain_vote_plans: vec![vote_plan.clone()],
            challenges,
        };

        let mut proposals = vec![];
        let mut rng = OsRng;

        for (index, proposal) in self.parameters.vote_plan.proposals().iter().enumerate() {
            let challenge_idx = rng.next_u32() as usize % self.parameters.challenges_count;
            let mut challenge = fund.challenges.get_mut(challenge_idx).unwrap();

            let proposal_template = template_generator.next_proposal();
            let proposal_funds = proposal_template.proposal_funds.parse().unwrap();
            let chain_vote_options = proposal_template.chain_vote_options.clone();

            if self.parameters.calculate_challenges_total_funds {
                challenge.rewards_total += proposal_funds;
            }

            let impact_score = proposal_template.proposal_impact_score_as_integer();

            let proposal = Proposal {
                internal_id: proposal_template.internal_id.parse().unwrap(),
                proposal_id: proposal.id().to_string(),
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
                proposal_impact_score: impact_score,
                proposal_files_url: proposal_template.files_url,
                proposer: Proposer {
                    proposer_name: proposal_template.proposer_name,
                    proposer_email: "".to_string(),
                    proposer_url: proposal_template.proposer_url,
                    proposer_relevant_experience: proposal_template.proposer_relevant_experience,
                },
                chain_proposal_id: proposal.id().to_string().as_bytes().to_vec(),
                chain_proposal_index: index as i64,
                chain_vote_options: self.parameters.vote_options.clone().unwrap_or_else(|| {
                    VoteOptions::parse_coma_separated_value(&chain_vote_options)
                }),
                chain_voteplan_id: chain_vote_plan_id.clone(),
                chain_vote_start_time: vote_plan.chain_vote_start_time,
                chain_vote_end_time: vote_plan.chain_vote_end_time,
                chain_committee_end_time: vote_plan.chain_committee_end_time,
                chain_voteplan_payload: vote_plan.chain_voteplan_payload.clone(),
                chain_vote_encryption_key: vote_plan.chain_vote_encryption_key.clone(),
                fund_id: fund.id,
                challenge_id: proposal_template
                    .challenge_id
                    .unwrap_or_else(|| challenge.id.to_string())
                    .parse()
                    .unwrap(),
                challenge_type: proposal_template.challenge_type,
                proposal_solution: proposal_template.proposal_solution,
                proposal_brief: proposal_template.proposal_brief,
                proposal_importance: proposal_template.proposal_importance,
                proposal_goal: proposal_template.proposal_goal,
                proposal_metrics: proposal_template.proposal_metrics,
            };

            proposals.push(proposal);
        }

        let challenges = fund.challenges.clone();

        Snapshot::new(
            vec![fund],
            proposals,
            challenges,
            generator.tokens(),
            vec![vote_plan],
        )
    }
}
