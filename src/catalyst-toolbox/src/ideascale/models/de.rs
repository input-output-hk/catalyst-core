use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "name")]
    pub title: String,
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
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fund {
    pub id: i32,
    pub name: String,
    #[serde(alias = "campaigns")]
    pub challenges: Vec<Challenge>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proposal {
    #[serde(alias = "id")]
    pub proposal_id: i32,
    pub proposal_category: Option<String>,
    #[serde(alias = "title")]
    pub proposal_title: String,
    #[serde(alias = "text")]
    pub proposal_summary: String,

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
    pub proposal_relevant_experience: String,
    #[serde(alias = "importance")]
    pub proposal_why: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl Funnel {
    pub fn is_community(&self) -> bool {
        self.title.contains("Challenge Setting")
    }
}
