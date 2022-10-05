use serde::Deserialize;

mod ada_rewards;
pub use ada_rewards::AdaRewards;

mod clean_string;
pub use clean_string::{clean_str, CleanString};

mod approval;
pub use approval::Approval;

mod challenge_title;
pub use challenge_title::ChallengeTitle;

#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    pub id: u32,
    #[serde(alias = "name")]
    pub title: ChallengeTitle,
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

    #[serde(alias = "flag")]
    pub approved: Approval,
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

impl Funnel {
    pub fn is_community(&self) -> bool {
        self.title.0.contains("Community Setting")
    }
}
