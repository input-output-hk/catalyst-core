use std::sync::{Arc, RwLock};

use vit_servicing_station_lib::db::models::{challenges::Challenge, proposals::ChallengeType};

use crate::mode::mock::{Configuration, Context, ContextLock};

#[macro_export]
macro_rules! make_context {
    (challenges = $challenges:expr) => {{
        use $crate::testing::mock_context::*;
        make_context_impl($challenges)
    }};
    () => {{
        make_context!(challenges = make_challenges())
    }};
}

pub fn make_challenge(key: i32) -> Challenge {
    Challenge {
        challenge_type: ChallengeType::Simple,
        internal_id: key,
        id: i32::MAX - key,
        title: format!("Challenge title {key}"),
        description: format!("Challenge description {key}"),
        rewards_total: 123,
        proposers_rewards: 100,
        fund_id: key,
        challenge_url: format!("https://some.challenge.org/{key}"),
        highlights: None,
    }
}

pub fn make_challenges() -> Vec<Challenge> {
    (0..10).map(make_challenge).collect()
}

pub fn make_context_impl(challenges: Vec<Challenge>) -> ContextLock {
    let mut context = Context::new(Configuration::default(), None).unwrap();
    *context.state_mut().vit_mut().challenges_mut() = challenges;
    Arc::new(RwLock::new(context))
}
