use serde::de::Error;
use serde::{Deserialize, Deserializer};

use std::collections::HashSet;
use std::ops::Deref;

lazy_static::lazy_static! {
    static ref DIRTY_CHARACTERS: HashSet<char> = ['*', '-', '/'].iter().copied().collect();
}

#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "name")]
    pub title: String,
    #[serde(alias = "tagline", deserialize_with = "deserialize_rewards")]
    pub rewards: String,
    pub description: String,
    #[serde(alias = "groupId")]
    pub fund_id: i32,
    #[serde(alias = "funnelId")]
    pub funnel_id: i32,
    #[serde(alias = "campaignUrl")]
    pub challenge_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Funnel {
    pub id: i32,
    #[serde(alias = "name")]
    pub title: CleanString,
    pub description: CleanString,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fund {
    pub id: i32,
    pub name: CleanString,
    #[serde(alias = "campaigns")]
    pub challenges: Vec<Challenge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proposal {
    #[serde(alias = "id")]
    pub proposal_id: i32,
    pub proposal_category: Option<CleanString>,
    #[serde(alias = "title")]
    pub proposal_title: CleanString,
    #[serde(alias = "text")]
    pub proposal_summary: CleanString,

    #[serde(alias = "url")]
    pub proposal_url: String,
    #[serde(default)]
    pub proposal_files_url: String,

    #[serde(alias = "customFieldsByKey")]
    pub custom_fields: ProposalCustomFieldsByKey,

    #[serde(alias = "authorInfo")]
    pub proposer: Proposer,

    #[serde(alias = "stageId")]
    pub stage_id: i32,

    #[serde(alias = "stageLabel")]
    pub stage_type: String,

    #[serde(alias = "campaignId")]
    pub challenge_id: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proposer {
    pub name: String,
    #[serde(alias = "email")]
    pub contact: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProposalCustomFieldsByKey {
    #[serde(alias = "ada_payment_address")]
    pub proposal_public_key: String,
    #[serde(alias = "requested_funds")]
    pub proposal_funds: String,
    #[serde(alias = "relevant_experience")]
    pub proposal_relevant_experience: CleanString,
    #[serde(alias = "importance")]
    pub proposal_why: Option<CleanString>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CleanString(#[serde(deserialize_with = "deserialize_clean_string")] String);

impl Funnel {
    pub fn is_community(&self) -> bool {
        self.title.contains("Challenge Setting")
    }
}

impl ToString for CleanString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Deref for CleanString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn deserialize_clean_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let mut rewards_str = String::deserialize(deserializer)?;
    rewards_str.retain(|c| !DIRTY_CHARACTERS.contains(&c));
    Ok(rewards_str)
}
fn deserialize_rewards<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let rewards_str = String::deserialize(deserializer)?;

    // input is not standarized, hack an early return if it is just 0 ada
    if rewards_str.starts_with("0 ada") {
        return Ok("0".to_string());
    }
    sscanf::scanf!(rewards_str.trim_end(), "${} in ada", String)
        // trim all . or , in between numbers
        .map(|mut s| {
            s.retain(|c: char| c.is_numeric());
            s
        })
        .ok_or_else(|| {
            D::Error::custom(&format!("Unable to read malformed value: {}", rewards_str))
        })
}
