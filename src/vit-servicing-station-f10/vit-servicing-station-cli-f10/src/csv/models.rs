use diesel::{ExpressionMethods, Insertable};
use serde::Deserialize;
use std::convert::TryInto;
use vit_servicing_station_lib_f10::db::models::challenges::{
    Challenge as DbChallenge, ChallengeHighlights,
};
use vit_servicing_station_lib_f10::db::models::community_advisors_reviews::{self, ReviewRanking};
use vit_servicing_station_lib_f10::db::models::proposals::{
    self, community_choice, simple, Category, ChallengeType, ProposalChallengeInfo, Proposer,
};
use vit_servicing_station_lib_f10::db::models::vote_options::VoteOptions;
use vit_servicing_station_lib_f10::db::schema::challenges;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "proposersRewards")]
    pub proposers_rewards: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeUrl")]
    pub challenge_url: String,
    pub highlights: Option<ChallengeHighlights>,
}

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
    #[serde(serialize_with = "vit_servicing_station_lib_f10::utils::serde::serialize_bin_as_str")]
    #[serde(
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_string_as_bytes"
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
        serialize_with = "vit_servicing_station_lib_f10::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib_f10::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib_f10::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_unix_timestamp_from_rfc3339"
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

impl Challenge {
    pub fn into_db_challenge_values(
        self,
    ) -> <DbChallenge as Insertable<challenges::table>>::Values {
        (
            challenges::id.eq(self.id),
            challenges::challenge_type.eq(self.challenge_type.to_string()),
            challenges::title.eq(self.title),
            challenges::description.eq(self.description),
            challenges::rewards_total.eq(self.rewards_total),
            challenges::proposers_rewards.eq(self.proposers_rewards),
            challenges::fund_id.eq(self.fund_id),
            challenges::challenge_url.eq(self.challenge_url),
            // This should always be a valid json
            challenges::highlights.eq(serde_json::to_string(&self.highlights).ok()),
        )
    }
}

impl Proposal {
    pub fn into_db_proposal_and_challenge_info(
        self,
        challenge_type: ChallengeType,
    ) -> Result<(proposals::Proposal, proposals::ProposalChallengeInfo), std::io::Error> {
        let proposal = proposals::Proposal {
            internal_id: self.internal_id,
            proposal_id: self.proposal_id,
            proposal_category: Category {
                category_id: self.category_id,
                category_name: self.category_name,
                category_description: self.category_description,
            },
            proposal_title: self.proposal_title,
            proposal_summary: self.proposal_summary,
            proposal_public_key: self.proposal_public_key,
            proposal_funds: self.proposal_funds,
            proposal_url: self.proposal_url,
            proposal_files_url: self.proposal_files_url,
            proposal_impact_score: self.proposal_impact_score,
            reviews_count: 0,
            proposer: Proposer {
                proposer_name: self.proposer_name,
                proposer_email: self.proposer_email,
                proposer_url: self.proposer_url,
                proposer_relevant_experience: self.proposer_relevant_experience,
            },
            chain_proposal_id: self.chain_proposal_id,
            chain_proposal_index: self.chain_proposal_index,
            chain_vote_options: VoteOptions::parse_coma_separated_value(&self.chain_vote_options),
            chain_voteplan_id: self.chain_voteplan_id,
            chain_vote_start_time: self.chain_vote_start_time,
            chain_vote_end_time: self.chain_vote_end_time,
            chain_committee_end_time: self.chain_committee_end_time,
            chain_voteplan_payload: self.chain_voteplan_payload,
            chain_vote_encryption_key: self.chain_vote_encryption_key,
            fund_id: self.fund_id,
            challenge_id: self.challenge_id,
        };

        let challenge_info = match challenge_type {
            ChallengeType::Simple | ChallengeType::Native => match self.proposal_solution {
                Some(proposal_solution) => {
                    ProposalChallengeInfo::Simple(simple::ChallengeInfo { proposal_solution })
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "cannot match simple proposal's challenge information fields:\
                            expected a value in `proposal_solution` column, found none",
                    ));
                }
            },
            ChallengeType::CommunityChoice => {
                match (
                    self.proposal_brief,
                    self.proposal_importance,
                    self.proposal_goal,
                    self.proposal_metrics,
                ) {
                    (
                        Some(proposal_brief),
                        Some(proposal_importance),
                        Some(proposal_goal),
                        Some(proposal_metrics),
                    ) => ProposalChallengeInfo::CommunityChoice(community_choice::ChallengeInfo {
                        proposal_brief,
                        proposal_importance,
                        proposal_goal,
                        proposal_metrics,
                    }),
                    values => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "cannot match community choice proposal's challenge information fields:\
                                expected values in columns `proposal_brief`, `proposal_importance`, `proposal_goal`, `proposal_metrics`, found: {:?}",
                                values
                            ),
                        ));
                    }
                }
            }
        };
        Ok((proposal, challenge_info))
    }
}

#[derive(Deserialize)]
pub struct AdvisorReview {
    id: i32,
    proposal_id: i32,
    assessor: String,
    impact_alignment_rating_given: i32,
    impact_alignment_note: String,
    feasibility_rating_given: i32,
    feasibility_note: String,
    auditability_rating_given: i32,
    auditability_note: String,
    #[serde(
        alias = "Excellent",
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_truthy_falsy"
    )]
    excellent: bool,
    #[serde(
        alias = "Good",
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_truthy_falsy"
    )]
    good: bool,
    #[serde(
        default,
        alias = "Filtered Out",
        deserialize_with = "vit_servicing_station_lib_f10::utils::serde::deserialize_truthy_falsy"
    )]
    filtered_out: bool,
}

impl TryInto<community_advisors_reviews::AdvisorReview> for AdvisorReview {
    type Error = std::io::Error;

    fn try_into(self) -> Result<community_advisors_reviews::AdvisorReview, Self::Error> {
        Ok(community_advisors_reviews::AdvisorReview {
            id: self.id,
            proposal_id: self.proposal_id,
            assessor: self.assessor,
            feasibility_note: self.feasibility_note,
            feasibility_rating_given: self.feasibility_rating_given,
            impact_alignment_note: self.impact_alignment_note,
            impact_alignment_rating_given: self.impact_alignment_rating_given,
            auditability_note: self.auditability_note,
            auditability_rating_given: self.auditability_rating_given,
            ranking: match (self.excellent, self.good, self.filtered_out) {
                (true, false, false) => ReviewRanking::Excellent,
                (false, true, false) => ReviewRanking::Good,
                (false, false, true) => ReviewRanking::FilteredOut,
                (false, false, false) => ReviewRanking::NA,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "expected one-hot encoding, found {}-{}-{}",
                            self.excellent, self.good, self.filtered_out
                        ),
                    ))
                }
            },
        })
    }
}
