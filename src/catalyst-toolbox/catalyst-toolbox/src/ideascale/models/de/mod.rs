use serde::{Deserialize, Deserializer};

mod ada_rewards;
pub use ada_rewards::AdaRewards;

mod clean_string;
pub use clean_string::{CleanString, clean_str};

#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    pub id: u32,
    #[serde(alias = "name", deserialize_with = "deserialize_clean_challenge_title")]
    pub title: String,
    #[serde(alias = "tagline")]
    pub rewards: AdaRewards,
    pub description: CleanString,
    #[serde(alias = "groupId")]
    pub fund_id: u32,
    #[serde(alias = "funnelId")]
    pub funnel_id: u32,
    #[serde(alias = "campaignUrl")]
    pub challenge_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Funnel {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: CleanString,
    pub description: CleanString,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fund {
    pub id: u32,
    pub name: CleanString,
    #[serde(alias = "campaigns")]
    pub challenges: Vec<Challenge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proposal {
    #[serde(alias = "id")]
    pub proposal_id: u32,
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
    pub stage_id: u32,

    #[serde(alias = "stageLabel")]
    pub stage_type: String,

    #[serde(alias = "campaignId")]
    pub challenge_id: u32,

    #[serde(alias = "flag", deserialize_with = "deserialize_approved")]
    pub approved: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proposer {
    pub name: String,
    #[serde(alias = "email")]
    pub contact: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProposalCustomFieldsByKey {
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Stage {
    #[serde(default)]
    pub label: String,
    #[serde(alias = "funnelId", default)]
    pub funnel_id: u32,
    #[serde(alias = "assessmentId", default)]
    pub assessment_id: u32,
}


fn deserialize_approved<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let approved = String::deserialize(deserializer)?;
    Ok(matches!(approved.as_str(), "approved"))
}
impl Funnel {
    pub fn is_community(&self) -> bool {
        self.title.as_ref().contains("Community Setting")
    }
}


fn deserialize_clean_challenge_title<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let mut rewards_str = String::deserialize(deserializer)?;
    // Remove leading `FX: `
    if rewards_str.starts_with('F') {
        if let Some(first_space) = rewards_str.find(' ') {
            let (_, content) = rewards_str.split_at(first_space + 1);
            rewards_str = content.to_string();
        }
    }
    Ok(rewards_str)
}

