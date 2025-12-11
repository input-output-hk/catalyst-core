use diesel::{ExpressionMethods, Insertable};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use vit_servicing_station_lib::db;
use vit_servicing_station_lib::db::models::challenges::{
    Challenge as DbChallenge, ChallengeHighlights,
};
use vit_servicing_station_lib::db::models::community_advisors_reviews::{self, ReviewRanking};
use vit_servicing_station_lib::db::models::goals::Goal;
use vit_servicing_station_lib::db::models::groups::Group;
use vit_servicing_station_lib::db::models::proposals::{
    self, community_choice, simple, Category, ChallengeType, ProposalChallengeInfo, Proposer,
};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;
use vit_servicing_station_lib::db::schema::challenges;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Fund {
    #[serde(default = "Default::default")]
    pub id: i32,
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_threshold: i64,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub fund_start_time: i64,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub fund_end_time: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub next_fund_start_time: i64,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub registration_snapshot_time: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub next_registration_snapshot_time: i64,
    #[serde(default = "Vec::new")]
    pub chain_vote_plans: Vec<Voteplan>,
    #[serde(default = "Vec::new")]
    pub challenges: Vec<Challenge>,
    #[serde(flatten, default)]
    pub stage_dates: FundStageDates,
    #[serde(default = "Vec::new")]
    pub goals: Vec<Goal>,
    #[serde(default)]
    pub results_url: String,
    #[serde(default)]
    pub survey_url: String,
    #[serde(default = "BTreeSet::new")]
    pub groups: BTreeSet<Group>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct FundStageDates {
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub insight_sharing_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub proposal_submission_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub refine_proposals_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub finalize_proposals_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub proposal_assessment_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub assessment_qa_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub snapshot_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub voting_start: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub voting_end: i64,
    #[serde(default)]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub tallying_end: i64,
}

impl TryInto<db::models::funds::FundStageDates> for FundStageDates {
    type Error = std::io::Error;

    fn try_into(self) -> Result<db::models::funds::FundStageDates, Self::Error> {
        Ok(db::models::funds::FundStageDates {
            insight_sharing_start: self.insight_sharing_start,
            proposal_submission_start: self.proposal_submission_start,
            refine_proposals_start: self.refine_proposals_start,
            finalize_proposals_start: self.finalize_proposals_start,
            proposal_assessment_start: self.proposal_assessment_start,
            assessment_qa_start: self.assessment_qa_start,
            snapshot_start: self.snapshot_start,
            voting_start: self.voting_start,
            voting_end: self.voting_end,
            tallying_end: self.tallying_end,
        })
    }
}
impl TryInto<db::models::funds::Fund> for Fund {
    type Error = std::io::Error;

    fn try_into(self) -> Result<db::models::funds::Fund, Self::Error> {
        Ok(db::models::funds::Fund {
            id: self.id,
            fund_name: self.fund_name,
            fund_goal: self.fund_goal,
            voting_power_threshold: self.voting_power_threshold,
            fund_start_time: self.fund_start_time,
            fund_end_time: self.fund_end_time,
            next_fund_start_time: self.next_fund_start_time,
            registration_snapshot_time: self.registration_snapshot_time,
            next_registration_snapshot_time: self.next_registration_snapshot_time,
            chain_vote_plans: self
                .chain_vote_plans
                .into_iter()
                .map(|x| x.try_into().unwrap())
                .collect(),
            challenges: self
                .challenges
                .into_iter()
                .map(|x| x.try_into().unwrap())
                .collect(),
            stage_dates: self.stage_dates.try_into().unwrap(),
            goals: self.goals,
            results_url: self.results_url,
            survey_url: self.survey_url,
            groups: self.groups,
        })
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Voteplan {
    pub id: i32,
    pub chain_voteplan_id: String,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub chain_vote_start_time: i64,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub chain_vote_end_time: i64,
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub chain_committee_end_time: i64,
    pub chain_voteplan_payload: String,
    pub chain_vote_encryption_key: String,
    pub fund_id: i32,
    #[serde(default)]
    pub token_identifier: String,
}

impl TryInto<db::models::voteplans::Voteplan> for Voteplan {
    type Error = std::io::Error;

    fn try_into(self) -> Result<db::models::voteplans::Voteplan, Self::Error> {
        Ok(db::models::voteplans::Voteplan {
            id: self.id,
            chain_voteplan_id: self.chain_voteplan_id.clone(),
            chain_vote_start_time: self.chain_vote_start_time,
            chain_vote_end_time: self.chain_vote_end_time,
            chain_committee_end_time: self.chain_committee_end_time,
            chain_voteplan_payload: self.chain_voteplan_payload,
            chain_vote_encryption_key: self.chain_vote_encryption_key,
            fund_id: self.fund_id,
            token_identifier: {
                if self.token_identifier.is_empty() {
                    self.chain_voteplan_id + "_token"
                } else {
                    self.token_identifier
                }
            },
        })
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    #[serde(default)]
    pub internal_id: i32,
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
    #[serde(default)]
    pub internal_id: i32,
    #[serde(default)]
    pub proposal_id: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub category_name: String,
    #[serde(default)]
    pub category_description: String,
    #[serde(default)]
    pub proposal_title: String,
    #[serde(default)]
    pub proposal_summary: String,
    #[serde(default)]
    pub proposal_public_key: String,
    #[serde(default)]
    pub proposal_funds: i64,
    #[serde(default)]
    pub proposal_url: String,
    #[serde(default)]
    pub proposal_files_url: String,
    #[serde(default)]
    pub proposal_impact_score: i64,
    #[serde(default)]
    pub proposer_name: String,
    #[serde(default)]
    pub proposer_email: String,
    #[serde(default)]
    pub proposer_url: String,
    #[serde(default)]
    pub proposer_relevant_experience: String,
    #[serde(alias = "chainProposalId")]
    #[serde(serialize_with = "vit_servicing_station_lib::utils::serde::serialize_bin_as_str")]
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_string_as_bytes"
    )]
    pub chain_proposal_id: Vec<u8>,
    #[serde(default)]
    pub chain_proposal_index: i64,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: String,
    #[serde(alias = "chainVoteStartTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime", default = "Default::default")]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    #[serde(deserialize_with = "crate::csv::deser::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload", default = "Default::default")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "chainVoteplanId", default = "Default::default")]
    pub chain_voteplan_id: String,
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
    #[serde(alias = "proposalExtraFields", default)]
    extra: Option<BTreeMap<String, String>>,
}

fn default_fund_id() -> i32 {
    -1
}

fn default_challenge_id() -> i32 {
    -1
}

impl TryInto<db::models::challenges::Challenge> for Challenge {
    type Error = std::io::Error;

    fn try_into(self) -> Result<db::models::challenges::Challenge, Self::Error> {
        Ok(db::models::challenges::Challenge {
            internal_id: self.internal_id,
            id: self.id,
            challenge_type: self.challenge_type,
            title: self.title,
            description: self.description,
            rewards_total: self.rewards_total,
            proposers_rewards: self.proposers_rewards,
            fund_id: self.fund_id,
            challenge_url: self.challenge_url,
            highlights: self.highlights,
        })
    }
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
            proposal_summary: self.proposal_summary.clone(),
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
            chain_vote_options: VoteOptions::parse_coma_separated_value(&self.chain_vote_options),
            chain_vote_start_time: self.chain_vote_start_time,
            chain_vote_end_time: self.chain_vote_end_time,
            chain_committee_end_time: self.chain_committee_end_time,
            chain_voteplan_payload: self.chain_voteplan_payload,
            chain_vote_encryption_key: self.chain_vote_encryption_key,
            fund_id: self.fund_id,
            challenge_id: self.challenge_id,
            extra: self.extra,
        };

        let challenge_info = match challenge_type {
            ChallengeType::Simple => match self.proposal_solution {
                Some(proposal_solution) => {
                    ProposalChallengeInfo::Simple(simple::ChallengeInfo { proposal_solution })
                }
                None => ProposalChallengeInfo::Simple(simple::ChallengeInfo {
                    proposal_solution: self.proposal_summary,
                }),
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
                    _values => {
                        ProposalChallengeInfo::CommunityChoice(community_choice::ChallengeInfo {
                            proposal_brief: proposal.proposal_summary.clone(),
                            proposal_importance: proposal.proposal_summary.clone(),
                            proposal_goal: proposal.proposal_summary.clone(),
                            proposal_metrics: proposal.proposal_summary.clone(),
                        })
                    }
                }
            }
        };
        Ok((proposal, challenge_info))
    }
}

#[derive(Deserialize)]
pub struct AdvisorReview {
    pub id: i32,
    pub proposal_id: i32,
    assessor: String,
    pub tag: Option<String>,
    pub rating_given: Option<i32>,
    pub note: Option<String>,
    #[serde(default)]
    impact_alignment_rating_given: i32,
    #[serde(default)]
    impact_alignment_note: String,
    #[serde(default)]
    feasibility_rating_given: i32,
    #[serde(default)]
    feasibility_note: String,
    #[serde(default)]
    auditability_rating_given: i32,
    #[serde(default)]
    auditability_note: String,
    #[serde(
        default,
        alias = "Excellent",
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_truthy_falsy"
    )]
    excellent: bool,
    #[serde(
        default,
        alias = "Good",
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_truthy_falsy"
    )]
    good: bool,
    #[serde(
        default,
        alias = "Filtered Out",
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_truthy_falsy"
    )]
    filtered_out: bool,
}

impl TryInto<community_advisors_reviews::AdvisorReview> for AdvisorReview {
    type Error = std::io::Error;

    fn try_into(self) -> Result<community_advisors_reviews::AdvisorReview, Self::Error> {
        if self.rating_given.is_some() {
            //legacy
            match self.tag.unwrap().as_str() {
                "Impact" => Ok(community_advisors_reviews::AdvisorReview {
                    id: self.id,
                    proposal_id: self.proposal_id,
                    assessor: self.assessor,
                    feasibility_note: "".to_string(),
                    feasibility_rating_given: 0,
                    impact_alignment_note: self.note.as_ref().unwrap().to_string(),
                    impact_alignment_rating_given: *self.rating_given.as_ref().unwrap(),
                    auditability_note: "".to_string(),
                    auditability_rating_given: 0,
                    ranking: ReviewRanking::NA,
                }),
                "Feasibility" => Ok(community_advisors_reviews::AdvisorReview {
                    id: self.id,
                    proposal_id: self.proposal_id,
                    assessor: self.assessor,
                    feasibility_note: self.note.as_ref().unwrap().to_string(),
                    feasibility_rating_given: *self.rating_given.as_ref().unwrap(),
                    impact_alignment_note: "".to_string(),
                    impact_alignment_rating_given: 0,
                    auditability_note: "".to_string(),
                    auditability_rating_given: 0,
                    ranking: ReviewRanking::NA,
                }),
                "Auditability" => Ok(community_advisors_reviews::AdvisorReview {
                    id: self.id,
                    proposal_id: self.proposal_id,
                    assessor: self.assessor,
                    feasibility_note: "".to_string(),
                    feasibility_rating_given: 0,
                    impact_alignment_note: "".to_string(),
                    impact_alignment_rating_given: 0,
                    auditability_note: self.note.as_ref().unwrap().to_string(),
                    auditability_rating_given: *self.rating_given.as_ref().unwrap(),
                    ranking: ReviewRanking::NA,
                }),
                _ => unreachable!(),
            }
        } else {
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
}
