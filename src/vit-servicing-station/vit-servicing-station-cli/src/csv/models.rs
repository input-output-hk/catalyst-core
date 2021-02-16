use serde::Deserialize;
use vit_servicing_station_lib::db::models::proposals;
use vit_servicing_station_lib::db::models::proposals::{Category, Proposer};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposal {
    #[serde(alias = "internalId")]
    pub internal_id: i32,
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    #[serde(alias = "categoryId", default = "Default::default")]
    pub category_id: String,
    #[serde(alias = "categoryName")]
    pub category_name: String,
    #[serde(alias = "categoryDescription", default = "Default::default")]
    pub category_description: String,
    #[serde(alias = "proposalTitle")]
    pub proposal_title: String,
    #[serde(alias = "proposalSummary")]
    pub proposal_summary: String,
    #[serde(alias = "proposalPublicKey")]
    pub proposal_public_key: String,
    #[serde(alias = "proposalFunds")]
    pub proposal_funds: i64,
    #[serde(alias = "proposalUrl")]
    pub proposal_url: String,
    #[serde(alias = "proposalFilesUrl")]
    pub proposal_files_url: String,
    #[serde(alias = "proposalImpactScore")]
    pub proposal_impact_score: i64,
    #[serde(alias = "proposerName")]
    pub proposer_name: String,
    #[serde(alias = "proposerEmail")]
    pub proposer_email: String,
    #[serde(alias = "proposerUrl")]
    pub proposer_url: String,
    #[serde(alias = "proposerRelevantExperience")]
    pub proposer_relevant_experience: String,
    #[serde(alias = "chainProposalId")]
    #[serde(serialize_with = "vit_servicing_station_lib::utils::serde::serialize_bin_as_str")]
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_string_as_bytes"
    )]
    pub chain_proposal_id: Vec<u8>,
    #[serde(alias = "chainProposalIndex")]
    pub chain_proposal_index: i64,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: String,
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainVoteStartTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload", default = "Default::default")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "chainVoteEncryptionKey", default = "Default::default")]
    pub chain_vote_encryption_key: String,
    #[serde(alias = "fundId", default = "default_fund_id")]
    pub fund_id: i32,
    #[serde(alias = "challengeId", default = "default_challenge_id")]
    pub challenge_id: i32,
    #[serde(alias = "proposalSolution", default)]
    proposal_solution: Option<String>,
    #[serde(alias = "proposalBrief", default)]
    proposal_brief: Option<String>,
    #[serde(alias = "proposalImportance", default)]
    proposal_importance: Option<String>,
    #[serde(alias = "proposalGoal", default)]
    proposal_goal: Option<String>,
    #[serde(alias = "proposalMetrics", default)]
    proposal_metrics: Option<String>,
}

fn default_fund_id() -> i32 {
    -1
}

fn default_challenge_id() -> i32 {
    -1
}

impl From<Proposal> for proposals::Proposal {
    fn from(proposal: Proposal) -> Self {
        Self {
            internal_id: proposal.internal_id,
            proposal_id: proposal.proposal_id,
            proposal_category: Category {
                category_id: proposal.category_id,
                category_name: proposal.category_name,
                category_description: proposal.category_description,
            },
            proposal_title: proposal.proposal_title,
            proposal_summary: proposal.proposal_summary,
            proposal_public_key: proposal.proposal_public_key,
            proposal_funds: proposal.proposal_funds,
            proposal_url: proposal.proposal_url,
            proposal_files_url: proposal.proposal_files_url,
            proposal_impact_score: proposal.proposal_impact_score,
            proposer: Proposer {
                proposer_name: proposal.proposer_name,
                proposer_email: proposal.proposer_email,
                proposer_url: proposal.proposer_url,
                proposer_relevant_experience: proposal.proposer_relevant_experience,
            },
            chain_proposal_id: proposal.chain_proposal_id,
            chain_proposal_index: proposal.chain_proposal_index,
            chain_vote_options: VoteOptions::parse_coma_separated_value(
                &proposal.chain_vote_options,
            ),
            chain_voteplan_id: proposal.chain_voteplan_id,
            chain_vote_start_time: proposal.chain_vote_start_time,
            chain_vote_end_time: proposal.chain_vote_end_time,
            chain_committee_end_time: proposal.chain_committee_end_time,
            chain_voteplan_payload: proposal.chain_voteplan_payload,
            chain_vote_encryption_key: proposal.chain_vote_encryption_key,
            fund_id: proposal.fund_id,
            challenge_id: proposal.challenge_id,
            proposal_solution: proposal.proposal_solution,
            proposal_brief: proposal.proposal_brief,
            proposal_importance: proposal.proposal_importance,
            proposal_goal: proposal.proposal_goal,
            proposal_metrics: proposal.proposal_metrics,
        }
    }
}
