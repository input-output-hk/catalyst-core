use chrono::{offset::Utc, Duration};
use vit_servicing_station_lib::{
    db::models::{
        api_tokens::APITokenData,
        funds::Fund,
        proposals::{Category, Proposal, Proposer},
        vote_options::VoteOptions,
        voteplans::Voteplan,
    },
    v0::api_token::APIToken,
};

use fake::faker::company::en::{Buzzword, CatchPhase, CompanyName, Industry};
use fake::{
    faker::lorem::en::*,
    faker::{chrono::en::DateTimeBetween, number::en::NumberWithFormat},
    faker::{internet::en::DomainSuffix, internet::en::SafeEmail, name::en::Name},
    Fake,
};
use rand::{rngs::OsRng, RngCore};
use std::{collections::HashMap, iter};

use chrono::DateTime;
type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct Snapshot {
    funds: Vec<Fund>,
    proposals: Vec<Proposal>,
    tokens: HashMap<String, APITokenData>,
}

impl Snapshot {
    pub fn funds(&self) -> Vec<Fund> {
        self.funds.clone()
    }

    pub fn proposals(&self) -> Vec<Proposal> {
        self.proposals.clone()
    }

    pub fn tokens(&self) -> HashMap<String, APITokenData> {
        self.tokens.clone()
    }

    pub fn proposal_by_id(&self, id: &str) -> Option<&Proposal> {
        self.proposals.iter().find(|x| x.proposal_id.eq(id))
    }

    pub fn fund_by_id(&self, id: i32) -> Option<&Fund> {
        self.funds.iter().find(|x| x.id == id)
    }

    pub fn any_token(&self) -> (String, APITokenData) {
        let (hash, token) = self.tokens.iter().next().clone().unwrap();
        (hash.to_string(), token.clone())
    }

    pub fn token_hash(&self) -> String {
        self.any_token().0
    }
}

pub struct Generator {
    id_generator: OsRng,
}

impl Default for Generator {
    fn default() -> Self {
        Generator::new()
    }
}

impl Generator {
    pub fn new() -> Self {
        Self {
            id_generator: OsRng,
        }
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

    pub fn token(&mut self) -> (String, APITokenData) {
        let data = self.bytes().to_vec();
        let token_creation_time = Utc::now() - Duration::days(1);
        let toket_expiry_time = Utc::now() + Duration::days(1);

        let token_data = APITokenData {
            token: APIToken::new(data.clone()),
            creation_time: token_creation_time.timestamp(),
            expire_time: toket_expiry_time.timestamp(),
        };
        (
            base64::encode_config(data, base64::URL_SAFE_NO_PAD),
            token_data,
        )
    }

    pub fn tokens(&mut self) -> HashMap<String, APITokenData> {
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
        let id = self.id_generator.next_u32() as i32;
        let (start, end, next) = self.consecutive_dates();

        Fund {
            id: id.abs(),
            fund_name: CatchPhase().fake::<String>(),
            fund_goal: Buzzword().fake::<String>(),
            voting_power_info: format!(">{}", NumberWithFormat("^###").fake::<String>()),
            voting_power_threshold: self.random_size() as i64,
            rewards_info: Sentence(3..5).fake::<String>(),
            fund_start_time: start.timestamp(),
            fund_end_time: end.timestamp(),
            next_fund_start_time: next.timestamp(),
            chain_vote_plans: vec![self.voteplan_with_fund_id(id.abs())],
        }
    }

    fn gen_http_address(&self) -> String {
        format!(
            "http://{}.{}",
            CompanyName()
                .fake::<String>()
                .to_lowercase()
                .replace(" ", "-"),
            DomainSuffix().fake::<String>()
        )
    }

    fn gen_single_proposal(&mut self, fund: &Fund) -> Proposal {
        let id = self.id_generator.next_u32() as i32;
        let proposal_url = self.gen_http_address();

        let voteplan = fund.chain_vote_plans.first().unwrap();

        Proposal {
            internal_id: id.abs(),
            proposal_id: id.abs().to_string(),
            proposal_category: Category {
                category_id: "".to_string(),
                category_name: Industry().fake::<String>(),
                category_description: "".to_string(),
            },
            proposal_title: CatchPhase().fake::<String>(),
            proposal_summary: CatchPhase().fake::<String>(),
            proposal_problem: Buzzword().fake::<String>(),
            proposal_solution: CatchPhase().fake::<String>(),
            proposal_public_key: self.hash(),
            proposal_funds: (self.id_generator.next_u64() as i64).abs(),
            proposal_url: proposal_url.to_string(),
            proposal_files_url: format!("{}/files", proposal_url),
            proposer: Proposer {
                proposer_name: Name().fake::<String>(),
                proposer_email: SafeEmail().fake::<String>(),
                proposer_url: self.gen_http_address(),
            },
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
            fund_id: fund.id,
        }
    }

    fn consecutive_dates(&self) -> (UtcDateTime, UtcDateTime, UtcDateTime) {
        let range_start_time = Utc::now() - Duration::days(10);
        let range_end_time = Utc::now() + Duration::days(10);
        let range_next_start_time = range_end_time + Duration::days(10);
        (
            DateTimeBetween(range_start_time, Utc::now()).fake::<UtcDateTime>(),
            DateTimeBetween(Utc::now(), range_end_time).fake::<UtcDateTime>(),
            DateTimeBetween(range_end_time, range_next_start_time).fake::<UtcDateTime>(),
        )
    }

    fn hash(&mut self) -> String {
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

    pub fn proposals(&mut self, funds: &[Fund]) -> Vec<Proposal> {
        funds.iter().map(|x| self.gen_single_proposal(x)).collect()
    }

    pub fn voteplan_with_fund_id(&mut self, fund_id: i32) -> Voteplan {
        let id = self.id_generator.next_u32() as i32;
        let (start, end, next) = self.consecutive_dates();

        Voteplan {
            id: id.abs(),
            chain_voteplan_id: self.hash(),
            chain_vote_start_time: start.timestamp(),
            chain_vote_end_time: end.timestamp(),
            chain_committee_end_time: next.timestamp(),
            chain_voteplan_payload: "bla".to_string(), //Sentence(3..5).fake::<String>(),
            fund_id,
        }
    }

    pub fn snapshot(&mut self) -> Snapshot {
        let funds = self.funds();
        let proposals = self.proposals(&funds);
        let tokens = self.tokens();

        Snapshot {
            funds,
            proposals,
            tokens,
        }
    }
}
