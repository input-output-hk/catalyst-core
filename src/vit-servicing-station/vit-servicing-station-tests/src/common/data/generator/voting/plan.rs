use crate::common::data::generator::{ArbitraryGenerator, Snapshot, ValidVotingDataContent};
use chain_impl_mockchain::certificate::VotePlan;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use rand::{rngs::OsRng, RngCore};
use vit_servicing_station_lib::db::models::{
    challenges::Challenge, funds::Fund, proposals::Proposal, vote_options::VoteOptions,
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
    pub vote_options: VoteOptions,
    pub challenges_count: usize,
    pub fund_id: i32,
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
            vote_options: VoteOptions::parse_coma_separated_value("blank,yes,no"),
            challenges_count: 4,
            fund_id: 1,
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
        self.vote_options = vote_options
    }

    pub fn set_fund_id(&mut self, fund_id: i32) {
        self.fund_id = fund_id;
    }
}

pub struct ValidVotePlanGenerator {
    parameters: ValidVotePlanParameters,
    content_generator: Box<dyn ValidVotingDataContent>,
}

impl ValidVotePlanGenerator {
    pub fn new(
        parameters: ValidVotePlanParameters,
        content_generator: Box<dyn ValidVotingDataContent>,
    ) -> Self {
        Self {
            parameters,
            content_generator,
        }
    }

    fn convert_to_vote_plan(vote_plan_def: &VotePlanDef) -> VotePlan {
        vote_plan_def.clone().into()
    }

    pub fn build(&mut self) -> Snapshot {
        let mut generator = ArbitraryGenerator::new();
        let vote_plan = Self::convert_to_vote_plan(&self.parameters.vote_plan);
        let chain_vote_plan_id = vote_plan.to_id().to_string();
        let threshold = self.parameters.voting_power_threshold.unwrap();
        let voting_start = self.parameters.voting_start.unwrap();
        let voting_tally_start = self.parameters.voting_tally_start.unwrap();
        let voting_tally_end = self.parameters.voting_tally_end.unwrap();
        let next_fund_start_time = self.parameters.next_fund_start_time.unwrap();
        let fund_id = self.parameters.fund_id;

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
            let challenge_data = self.content_generator.next_challenge();
            Some(Challenge {
                id: generator.id().abs(),
                title: challenge_data.title,
                description: challenge_data.description,
                rewards_total: 0,
                fund_id,
                challenge_url: challenge_data.challenge_url,
            })
        })
        .take(count)
        .collect();

        let naive = NaiveDateTime::from_timestamp(voting_start, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        let fund_content = self.content_generator.next_fund();
        let mut fund = Fund {
            id: fund_id,
            fund_name: self.parameters.vote_plan.alias(),
            fund_goal: fund_content.goal,
            voting_power_info: datetime.to_rfc3339_opts(SecondsFormat::Secs, true),
            voting_power_threshold: threshold,
            rewards_info: fund_content.rewards_info,
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

            let proposal_content = self.content_generator.next_proposal();
            let proposal_funds = proposal_content.funds;

            challenge.rewards_total += proposal_funds;

            let proposal = Proposal {
                internal_id: index as i32,
                proposal_id: proposal.id().to_string(),
                proposal_category: proposal_content.category,
                proposal_title: proposal_content.title,
                proposal_summary: proposal_content.summary,
                proposal_problem: proposal_content.problem,
                proposal_solution: proposal_content.solution,
                proposal_public_key: generator.hash(),
                proposal_funds,
                proposal_url: proposal_content.url,
                proposal_impact_score: proposal_content.impact_score,
                proposal_files_url: proposal_content.files_url,
                proposer: proposal_content.proposer,
                chain_proposal_id: proposal.id().to_string().as_bytes().to_vec(),
                chain_proposal_index: index as i64,
                chain_vote_options: self.parameters.vote_options.clone(),
                chain_voteplan_id: chain_vote_plan_id.clone(),
                chain_vote_start_time: vote_plan.chain_vote_start_time,
                chain_vote_end_time: vote_plan.chain_vote_end_time,
                chain_committee_end_time: vote_plan.chain_committee_end_time,
                chain_voteplan_payload: vote_plan.chain_voteplan_payload.clone(),
                chain_vote_encryption_key: vote_plan.chain_vote_encryption_key.clone(),
                fund_id: fund.id,
                challenge_id: challenge.id,
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
