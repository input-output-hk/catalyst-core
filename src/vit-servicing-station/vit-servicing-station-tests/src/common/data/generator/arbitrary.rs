use crate::common::data::Snapshot;
use chrono::{offset::Utc, Duration};
use vit_servicing_station_lib::{
    db::models::{
        api_tokens::ApiTokenData,
        challenges::Challenge,
        funds::Fund,
        proposals::{Category, ChallengeType, Proposal, Proposer},
        vote_options::VoteOptions,
        voteplans::Voteplan,
    },
    v0::api_token::ApiToken,
};

use fake::{
    faker::chrono::en::DateTimeBetween,
    faker::company::en::{Buzzword, CatchPhase, CompanyName, Industry},
    faker::{internet::en::DomainSuffix, internet::en::SafeEmail, name::en::Name},
    Fake,
};
use rand::{rngs::OsRng, RngCore};
use std::{collections::HashMap, iter};

use chrono::DateTime;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::{
    community_choice, simple, FullProposalInfo, ProposalChallengeInfo,
};

type UtcDateTime = DateTime<Utc>;

struct FundDateTimes {
    start: UtcDateTime,
    end: UtcDateTime,
    next: UtcDateTime,
    snapshot: UtcDateTime,
}

struct VoteplanDateTimes {
    start: UtcDateTime,
    end: UtcDateTime,
    tally: UtcDateTime,
}

#[derive(Clone)]
pub struct ArbitraryGenerator {
    id_generator: OsRng,
}

impl Default for ArbitraryGenerator {
    fn default() -> Self {
        ArbitraryGenerator::new()
    }
}

impl ArbitraryGenerator {
    pub fn new() -> Self {
        Self {
            id_generator: OsRng,
        }
    }

    pub fn random_index(&mut self, limit: usize) -> usize {
        (self.id_generator.next_u32() as usize) % limit
    }

    fn random_size(&mut self) -> usize {
        (self.id_generator.next_u32() as usize) % 100 + 1
    }

    fn bytes(&mut self) -> [u8; 32] {
        let mut random_bytes: [u8; 32] = [0; 32];
        self.id_generator.fill_bytes(&mut random_bytes);
        random_bytes
    }

    pub fn token_hash(&mut self) -> String {
        base64::encode_config(self.bytes().to_vec(), base64::URL_SAFE_NO_PAD)
    }

    pub fn id(&mut self) -> i32 {
        self.id_generator.next_u32() as i32
    }

    pub fn token(&mut self) -> (String, ApiTokenData) {
        let data = self.bytes().to_vec();
        let token_creation_time = Utc::now() - Duration::days(1);
        let toket_expiry_time = Utc::now() + Duration::days(1);

        let token_data = ApiTokenData {
            token: ApiToken::new(data.clone()),
            creation_time: token_creation_time.timestamp(),
            expire_time: toket_expiry_time.timestamp(),
        };
        (
            base64::encode_config(data, base64::URL_SAFE_NO_PAD),
            token_data,
        )
    }

    pub fn tokens(&mut self) -> HashMap<String, ApiTokenData> {
        let size = self.random_size() % 10 + 2;
        iter::from_fn(|| Some(self.token())).take(size).collect()
    }

    pub fn funds(&mut self) -> Vec<Fund> {
        let size = self.random_size();
        iter::from_fn(|| Some(self.gen_single_fund()))
            .take(size)
            .collect()
    }

    fn gen_single_fund(&mut self) -> Fund {
        let id = self.id();
        let dates = self.fund_date_times();

        Fund {
            id: id.abs(),
            fund_name: CatchPhase().fake::<String>(),
            fund_goal: Buzzword().fake::<String>(),
            fund_start_time: dates.start.timestamp(),
            voting_power_threshold: self.id_generator.next_u64() as i64,
            fund_end_time: dates.end.timestamp(),
            next_fund_start_time: dates.next.timestamp(),
            registration_snapshot_time: dates.snapshot.timestamp(),
            chain_vote_plans: vec![self.voteplan_with_fund_id(id.abs())],
            challenges: self.challenges_with_fund_id(id.abs()),
        }
    }

    pub fn gen_http_address(&self) -> String {
        format!(
            "http://{}.{}",
            CompanyName()
                .fake::<String>()
                .to_lowercase()
                .replace(" ", "-"),
            DomainSuffix().fake::<String>()
        )
    }

    pub fn proposer(&mut self) -> Proposer {
        Proposer {
            proposer_relevant_experience: Buzzword().fake::<String>(),
            proposer_name: Name().fake::<String>(),
            proposer_email: SafeEmail().fake::<String>(),
            proposer_url: self.gen_http_address(),
        }
    }
    // impact score [1.00-4.99]
    pub fn impact_score(&mut self) -> f64 {
        ((self.id_generator.next_u64() % 400 + 100) / 100) as f64
    }

    pub fn proposal_category(&mut self) -> Category {
        Category {
            category_id: "".to_string(),
            category_name: Industry().fake::<String>(),
            category_description: "".to_string(),
        }
    }

    pub fn proposal_fund(&mut self) -> i64 {
        (self.id_generator.next_u64() % 200_000 + 5000) as i64
    }

    pub fn challenge_type(&mut self) -> ChallengeType {
        match self.id_generator.next_u32() % 2 {
            0 => ChallengeType::Simple,
            1 => ChallengeType::CommunityChoice,
            _ => unreachable!(),
        }
    }

    pub fn proposals_challenge_info(
        &mut self,
        challenge_type: &ChallengeType,
    ) -> ProposalChallengeInfo {
        match challenge_type {
            ChallengeType::Simple => ProposalChallengeInfo::Simple(simple::ChallengeInfo {
                proposal_solution: CatchPhase().fake::<String>(),
            }),
            ChallengeType::CommunityChoice => {
                ProposalChallengeInfo::CommunityChoice(community_choice::ChallengeInfo {
                    proposal_brief: CatchPhase().fake::<String>(),
                    proposal_importance: CatchPhase().fake::<String>(),
                    proposal_goal: CatchPhase().fake::<String>(),
                    proposal_metrics: CatchPhase().fake::<String>(),
                })
            }
        }
    }

    fn gen_single_proposal(&mut self, fund: &Fund) -> FullProposalInfo {
        let id = self.id_generator.next_u32() as i32;
        let proposal_url = self.gen_http_address();

        let voteplan = fund.chain_vote_plans.first().unwrap();
        let challenge = fund.challenges.first().unwrap();
        let challenge_id = challenge.id;
        let challenge_info = self.proposals_challenge_info(&challenge.challenge_type);
        let proposal = Proposal {
            internal_id: id.abs(),
            proposal_id: id.abs().to_string(),
            proposal_category: self.proposal_category(),
            proposal_title: CatchPhase().fake::<String>(),
            proposal_summary: CatchPhase().fake::<String>(),
            proposal_public_key: self.hash(),
            proposal_funds: self.proposal_fund(),
            proposal_url: proposal_url.to_string(),
            proposal_impact_score: (self.impact_score() * 100f64) as i64,
            proposal_files_url: format!("{}/files", proposal_url),
            proposer: self.proposer(),
            chain_proposal_id: self.hash().as_bytes().to_vec(),
            chain_proposal_index: self.id_generator.next_u32() as i64,
            chain_vote_options: VoteOptions::parse_coma_separated_value("b,a,r"),
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
        FundDateTimes {
            start,
            end,
            next,
            snapshot,
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

    pub fn hash(&mut self) -> String {
        let mut hash = [0u8; 32];
        self.id_generator.fill_bytes(&mut hash);
        base64::encode(hash)
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
            chain_voteplan_id: self.hash(),
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

        vec![
            Challenge {
                id: simple_id.abs(),
                challenge_type: ChallengeType::Simple,
                title: CatchPhase().fake::<String>(),
                description: Buzzword().fake::<String>(),
                rewards_total: 100500,
                proposers_rewards: 100000,
                fund_id,
                challenge_url: self.gen_http_address(),
            },
            Challenge {
                id: community_choice_id.abs(),
                challenge_type: ChallengeType::CommunityChoice,
                title: CatchPhase().fake::<String>(),
                description: Buzzword().fake::<String>(),
                rewards_total: 100500,
                proposers_rewards: 100000,
                fund_id,
                challenge_url: self.gen_http_address(),
            },
        ]
    }

    pub fn challenge_with_fund_id(&mut self, fund_id: i32) -> Challenge {
        let id = self.id_generator.next_u32() as i32;

        Challenge {
            id: id.abs(),
            challenge_type: self.challenge_type(),
            title: CatchPhase().fake::<String>(),
            description: Buzzword().fake::<String>(),
            rewards_total: 100500,
            proposers_rewards: 100000,
            fund_id,
            challenge_url: self.gen_http_address(),
        }
    }

    pub fn review_with_proposal_id(&mut self, proposal_id: i32) -> AdvisorReview {
        let id = self.id_generator.next_u32() as i32;

        AdvisorReview {
            id: id.abs(),
            proposal_id,
            rating_given: (self.id_generator.next_u32() % 500) as i32,
            assessor: Name().fake::<String>(),
            note: fake::faker::lorem::en::Sentence(0..100).fake::<String>(),
        }
    }

    pub fn snapshot(&mut self) -> Snapshot {
        let funds = self.funds();
        let voteplans = self.voteplans(&funds);
        let challenges = self.challenges(&funds);
        let proposals = self.proposals(&funds);
        let reviews = self.advisor_reviews(&proposals);
        let tokens = self.tokens();

        Snapshot::new(funds, proposals, challenges, tokens, voteplans, reviews)
    }
}
