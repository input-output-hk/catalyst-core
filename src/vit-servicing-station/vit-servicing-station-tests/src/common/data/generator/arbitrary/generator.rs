use crate::common::data::{CurrentFund, ValidVotePlanParameters};
use chain_impl_mockchain::testing::scenario::template::ProposalDefBuilder;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::testing::scenario::template::VotePlanDefBuilder;
use fake::faker::name::en::Name;
use fake::Fake;
use rand::{rngs::OsRng, RngCore};
use std::{collections::HashMap, iter};
use time::{Duration, OffsetDateTime};
use vit_servicing_station_lib::{db::models::api_tokens::ApiTokenData, v0::api_token::ApiToken};

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

    pub fn random_size(&mut self) -> usize {
        (self.id_generator.next_u32() as usize) % 100 + 1
    }

    pub fn bytes(&mut self) -> [u8; 32] {
        let mut random_bytes: [u8; 32] = [0; 32];
        self.id_generator.fill_bytes(&mut random_bytes);
        random_bytes
    }

    pub fn next_u32(&mut self) -> u32 {
        self.id_generator.next_u32()
    }

    pub fn next_u64(&mut self) -> u64 {
        self.id_generator.next_u64()
    }

    pub fn token_hash(&mut self) -> String {
        base64::encode_config(self.bytes(), base64::URL_SAFE_NO_PAD)
    }

    pub fn id(&mut self) -> i32 {
        self.id_generator.next_u32() as i32
    }

    pub fn token(&mut self) -> (String, ApiTokenData) {
        let data = self.bytes().to_vec();
        let token_creation_time = OffsetDateTime::now_utc() - Duration::days(1);
        let toket_expiry_time = OffsetDateTime::now_utc() + Duration::days(1);

        let token_data = ApiTokenData {
            token: ApiToken::new(data.clone()),
            creation_time: token_creation_time.unix_timestamp(),
            expire_time: toket_expiry_time.unix_timestamp(),
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

    pub fn hash(&mut self) -> String {
        let mut hash = [0u8; 32];
        self.id_generator.fill_bytes(&mut hash);
        base64::encode(hash)
    }

    pub fn vote_plan_def(&mut self) -> VotePlanDef {
        let mut vote_plan_builder = VotePlanDefBuilder::new("fund_x");
        vote_plan_builder.owner(&Name().fake::<String>());
        vote_plan_builder.vote_phases(1, 2, 3);

        for _ in 0..(self.next_u32() % 245 + 10) {
            let mut proposal_builder = ProposalDefBuilder::new(
                chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
            );
            proposal_builder.options(3);
            proposal_builder.action_off_chain();
            vote_plan_builder.with_proposal(&mut proposal_builder);
        }

        vote_plan_builder.build()
    }

    pub fn vote_plan_def_collection(&mut self) -> Vec<VotePlanDef> {
        let len = (self.next_u32() % 10 + 1) as usize;
        std::iter::from_fn(|| Some(self.vote_plan_def()))
            .take(len)
            .collect()
    }

    pub fn valid_vote_plan_parameters(&mut self) -> ValidVotePlanParameters {
        CurrentFund::new(self.vote_plan_def_collection(), Default::default()).into()
    }
}
