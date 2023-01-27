use super::vote_options;
use crate::db::models::vote_options::VoteOptions;
use crate::db::schema::proposals_voteplans;
use crate::db::{schema::proposals, views_schema::full_proposals_info};
use diesel::backend::Backend;
use diesel::sql_types::{BigInt, Binary, Integer, Text};
use diesel::types::FromSql;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

pub mod community_choice;
pub mod simple;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Category {
    #[serde(alias = "categoryId", default = "Default::default")]
    pub category_id: String,
    #[serde(alias = "categoryName")]
    pub category_name: String,
    #[serde(alias = "categoryDescription", default = "Default::default")]
    pub category_description: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposer {
    #[serde(alias = "proposerName")]
    pub proposer_name: String,
    #[serde(alias = "proposerEmail")]
    pub proposer_email: String,
    #[serde(alias = "proposerUrl")]
    pub proposer_url: String,
    #[serde(alias = "proposerRelevantExperience")]
    pub proposer_relevant_experience: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengeType {
    Simple,
    CommunityChoice,
}

impl std::str::FromStr for ChallengeType {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(ChallengeType::Simple),
            "community-choice" => Ok(ChallengeType::CommunityChoice),
            s => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected any of [simple | community-choice], found: {}", s),
            )),
        }
    }
}

impl std::fmt::Display for ChallengeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // should be implemented and safe to unwrap here
        let repr = serde_json::to_string(&self).unwrap();
        write!(f, "{}", repr.trim_matches('"'))
    }
}

pub type ProposalExtraFields = BTreeMap<String, String>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposal {
    #[serde(alias = "internalId")]
    pub internal_id: i32,
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    #[serde(alias = "category")]
    pub proposal_category: Category,
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
    pub proposer: Proposer,
    #[serde(alias = "chainProposalId")]
    #[serde(serialize_with = "crate::utils::serde::serialize_bin_as_str")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_string_as_bytes")]
    pub chain_proposal_id: Vec<u8>,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: VoteOptions,
    #[serde(alias = "chainVoteStartTime", default = "Default::default")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime", default = "Default::default")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime", default = "Default::default")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "chainVoteEncryptionKey")]
    pub chain_vote_encryption_key: String,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeId")]
    pub challenge_id: i32,
    #[serde(alias = "reviewsCount")]
    pub reviews_count: i32,
    #[serde(alias = "extraFields")]
    pub extra: Option<ProposalExtraFields>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ProposalChallengeInfo {
    Simple(simple::ChallengeInfo),
    CommunityChoice(community_choice::ChallengeInfo),
}

#[derive(Serialize, Deserialize)]
struct SerdeProposalChallengeInfo {
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    simple: Option<simple::ChallengeInfo>,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    community: Option<community_choice::ChallengeInfo>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct FullProposalInfo {
    #[serde(flatten)]
    pub proposal: Proposal,
    #[serde(flatten)]
    pub challenge_info: ProposalChallengeInfo,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    #[serde(flatten)]
    pub voteplan: ProposalVotePlanCommon,
    #[serde(alias = "groupId")]
    pub group_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ProposalVotePlanCommon {
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainProposalIndex")]
    pub chain_proposal_index: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ProposalVotePlan {
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    #[serde(flatten)]
    pub common: ProposalVotePlanCommon,
}

impl Serialize for ProposalChallengeInfo {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let serde_data: SerdeProposalChallengeInfo = self.clone().into();
        serde_data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProposalChallengeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let serde_data: SerdeProposalChallengeInfo =
            SerdeProposalChallengeInfo::deserialize(deserializer)?;
        serde_data.try_into().map_err(|_| {
            <D as Deserializer<'de>>::Error::custom("Invalid data for ProposalChallengeInfo")
        })
    }
}

type FullProposalsInfoRow = (
    // 0 -> id
    i32,
    // 1 -> proposal_id
    String,
    // 2 -> proposal_category
    String,
    // 3 -> proposal_title
    String,
    // 4 -> proposal_summary
    String,
    // 5 -> proposal_public_key
    String,
    // 6 -> proposal_funds
    i64,
    // 7 -> proposal_url
    String,
    // 8 -> proposal_files_url
    String,
    // 9 -> proposal_impact_score
    i64,
    // 10 -> proposer_name
    String,
    // 11 -> proposer_contact
    String,
    // 12 -> proposer_url
    String,
    // 13 -> proposer_relevant_experience
    String,
    // 14 -> chain_proposal_id
    Vec<u8>,
    // 15 -> chain_vote_options
    String,
    // 16 -> challenge_id
    i32,
    // 17 -> extra_fields
    Option<String>,
    // 18 -> reviews_count
    i32,
    // 19 -> chain_vote_start_time
    i64,
    // 20 -> chain_vote_end_time
    i64,
    // 21 -> chain_committee_end_time
    i64,
    // 22 -> chain_voteplan_payload
    String,
    // 23 -> chain_vote_encryption_key
    String,
    // 24 -> fund_id
    i32,
    // 25 -> challenge_type
    String,
    // 26 -> proposal_solution
    Option<String>,
    // 27 -> proposal_brief
    Option<String>,
    // 28 -> proposal_importance
    Option<String>,
    // 29 -> proposal_goal
    Option<String>,
    // 30 -> proposal_metrics
    Option<String>,
    // 31 -> chain_proposal_index
    i64,
    // 32 -> chain_voteplan_id
    String,
    // 33 -> group_id
    String,
);

impl<DB: Backend> Queryable<full_proposals_info::SqlType, DB> for Proposal
where
    i32: FromSql<Integer, DB>,
    i64: FromSql<BigInt, DB>,
    String: FromSql<Text, DB>,
    Vec<u8>: FromSql<Binary, DB>,
{
    type Row = FullProposalsInfoRow;

    fn build(row: Self::Row) -> Self {
        Proposal {
            internal_id: row.0,
            proposal_id: row.1,
            proposal_category: Category {
                category_id: "".to_string(),
                category_name: row.2,
                category_description: "".to_string(),
            },
            proposal_title: row.3,
            proposal_summary: row.4,
            proposal_public_key: row.5,
            proposal_funds: row.6,
            proposal_url: row.7,
            proposal_files_url: row.8,
            proposal_impact_score: row.9,
            proposer: Proposer {
                proposer_name: row.10,
                proposer_email: row.11,
                proposer_url: row.12,
                proposer_relevant_experience: row.13,
            },
            chain_proposal_id: row.14,
            chain_vote_options: vote_options::VoteOptions::parse_coma_separated_value(&row.15),
            chain_vote_start_time: row.19,
            chain_vote_end_time: row.20,
            chain_committee_end_time: row.21,
            chain_voteplan_payload: row.22,
            chain_vote_encryption_key: row.23,
            fund_id: row.24,
            challenge_id: row.16,
            reviews_count: row.18,
            extra: row.17.map(|s| {
                serde_json::from_str(&s).expect("invalid value for proposal extra_fields")
            }),
        }
    }
}

impl<DB: Backend> Queryable<full_proposals_info::SqlType, DB> for ProposalVotePlan
where
    i32: FromSql<Integer, DB>,
    i64: FromSql<BigInt, DB>,
    String: FromSql<Text, DB>,
    Vec<u8>: FromSql<Binary, DB>,
{
    type Row = FullProposalsInfoRow;

    fn build(row: Self::Row) -> Self {
        ProposalVotePlan {
            proposal_id: row.1,
            common: ProposalVotePlanCommon {
                chain_proposal_index: row.31,
                chain_voteplan_id: row.32,
            },
        }
    }
}

impl<DB: Backend> Queryable<full_proposals_info::SqlType, DB> for FullProposalInfo
where
    i32: FromSql<Integer, DB>,
    i64: FromSql<BigInt, DB>,
    String: FromSql<Text, DB>,
    Vec<u8>: FromSql<Binary, DB>,
{
    type Row = FullProposalsInfoRow;

    fn build(row: Self::Row) -> Self {
        let challenge_type = row.25.parse().unwrap();
        // It should be safe to unwrap this values here if DB is sanitized and hence tables have data
        // relative to the challenge type.
        let challenge_info = match challenge_type {
            ChallengeType::Simple => ProposalChallengeInfo::Simple(simple::ChallengeInfo {
                proposal_solution: row.26.clone().unwrap(),
            }),
            ChallengeType::CommunityChoice => {
                ProposalChallengeInfo::CommunityChoice(community_choice::ChallengeInfo {
                    proposal_brief: row.27.clone().unwrap(),
                    proposal_importance: row.28.clone().unwrap(),
                    proposal_goal: row.29.clone().unwrap(),
                    proposal_metrics: row.30.clone().unwrap(),
                })
            }
        };

        let voteplan = ProposalVotePlanCommon {
            chain_proposal_index: row.31,
            chain_voteplan_id: row.32.clone(),
        };

        let group_id = row.33.clone();

        FullProposalInfo {
            proposal: Proposal::build(row),
            challenge_info,
            challenge_type,
            voteplan,
            group_id,
        }
    }
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<proposals::table> for Proposal {
    type Values = (
        diesel::dsl::Eq<proposals::proposal_id, String>,
        diesel::dsl::Eq<proposals::proposal_category, String>,
        diesel::dsl::Eq<proposals::proposal_title, String>,
        diesel::dsl::Eq<proposals::proposal_summary, String>,
        diesel::dsl::Eq<proposals::proposal_public_key, String>,
        diesel::dsl::Eq<proposals::proposal_funds, i64>,
        diesel::dsl::Eq<proposals::proposal_url, String>,
        diesel::dsl::Eq<proposals::proposal_files_url, String>,
        diesel::dsl::Eq<proposals::proposal_impact_score, i64>,
        diesel::dsl::Eq<proposals::proposer_name, String>,
        diesel::dsl::Eq<proposals::proposer_contact, String>,
        diesel::dsl::Eq<proposals::proposer_url, String>,
        diesel::dsl::Eq<proposals::proposer_relevant_experience, String>,
        diesel::dsl::Eq<proposals::chain_proposal_id, Vec<u8>>,
        diesel::dsl::Eq<proposals::chain_vote_options, String>,
        diesel::dsl::Eq<proposals::challenge_id, i32>,
        diesel::dsl::Eq<proposals::extra, Option<String>>,
    );

    fn values(self) -> Self::Values {
        (
            proposals::proposal_id.eq(self.proposal_id),
            proposals::proposal_category.eq(self.proposal_category.category_name),
            proposals::proposal_title.eq(self.proposal_title),
            proposals::proposal_summary.eq(self.proposal_summary),
            proposals::proposal_public_key.eq(self.proposal_public_key),
            proposals::proposal_funds.eq(self.proposal_funds),
            proposals::proposal_url.eq(self.proposal_url),
            proposals::proposal_files_url.eq(self.proposal_files_url),
            proposals::proposal_impact_score.eq(self.proposal_impact_score),
            proposals::proposer_name.eq(self.proposer.proposer_name),
            proposals::proposer_contact.eq(self.proposer.proposer_email),
            proposals::proposer_url.eq(self.proposer.proposer_url),
            proposals::proposer_relevant_experience.eq(self.proposer.proposer_relevant_experience),
            proposals::chain_proposal_id.eq(self.chain_proposal_id),
            proposals::chain_vote_options.eq(self.chain_vote_options.as_csv_string()),
            proposals::challenge_id.eq(self.challenge_id),
            proposals::extra.eq(self.extra.map(|h| serde_json::to_string(&h).unwrap())),
        )
    }
}

impl Insertable<proposals_voteplans::table> for ProposalVotePlan {
    type Values = (
        diesel::dsl::Eq<proposals_voteplans::proposal_id, String>,
        diesel::dsl::Eq<proposals_voteplans::chain_proposal_index, i64>,
        diesel::dsl::Eq<proposals_voteplans::chain_voteplan_id, String>,
    );

    fn values(self) -> Self::Values {
        (
            proposals_voteplans::proposal_id.eq(self.proposal_id),
            proposals_voteplans::chain_proposal_index.eq(self.common.chain_proposal_index),
            proposals_voteplans::chain_voteplan_id.eq(self.common.chain_voteplan_id),
        )
    }
}

struct SerdeToProposalChallengeInfoError;

impl TryFrom<SerdeProposalChallengeInfo> for ProposalChallengeInfo {
    type Error = SerdeToProposalChallengeInfoError;

    fn try_from(data: SerdeProposalChallengeInfo) -> Result<Self, Self::Error> {
        let SerdeProposalChallengeInfo { simple, community } = data;
        match (simple, community) {
            (None, None) | (Some(_), Some(_)) => Err(SerdeToProposalChallengeInfoError),
            (Some(simple), None) => Ok(ProposalChallengeInfo::Simple(simple)),
            (None, Some(community_challenge)) => {
                Ok(ProposalChallengeInfo::CommunityChoice(community_challenge))
            }
        }
    }
}

impl From<ProposalChallengeInfo> for SerdeProposalChallengeInfo {
    fn from(data: ProposalChallengeInfo) -> Self {
        match data {
            ProposalChallengeInfo::Simple(simple) => SerdeProposalChallengeInfo {
                simple: Some(simple),
                community: None,
            },
            ProposalChallengeInfo::CommunityChoice(community) => SerdeProposalChallengeInfo {
                simple: None,
                community: Some(community),
            },
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        models::{
            challenges::{
                test::{get_test_challenge_with_fund_id, populate_db_with_challenge_conn},
                Challenge,
            },
            groups::Group,
            vote_options::VoteOptions,
        },
        schema::{
            proposal_community_choice_challenge, proposal_simple_challenge, proposals,
            proposals_voteplans, voteplans,
        },
        DbConnection, DbConnectionPool,
    };
    use diesel::{ExpressionMethods, RunQueryDsl};
    use time::OffsetDateTime;

    pub fn add_test_proposal_and_challenge(
        key: i32,
        conn: &DbConnection,
    ) -> (FullProposalInfo, Challenge) {
        let mut proposal = get_test_proposal(format!("{key}"));
        proposal.proposal.internal_id = key;
        proposal.proposal.proposal_id = key.to_string();
        proposal.proposal.proposal_title = format!("proposal number {key}");
        proposal.voteplan.chain_voteplan_id = format!("voteplan_id_{key}");

        let mut challenge = get_test_challenge_with_fund_id(proposal.proposal.fund_id);
        challenge.title = format!("challenge {key}");
        challenge.description = format!("challenge description {key}");
        challenge.id = key;

        populate_db_with_proposal_conn(&proposal, conn);
        populate_db_with_challenge_conn(&challenge, conn);

        (proposal, challenge)
    }

    pub fn get_test_proposal(group_id: impl Into<String>) -> FullProposalInfo {
        const CHALLENGE_ID: i32 = 9001;

        let internal_id = 1;
        FullProposalInfo {
            proposal: Proposal {
                internal_id,
                proposal_id: "1".to_string(),
                proposal_category: Category {
                    category_id: "".to_string(),
                    category_name: "foo_category_name".to_string(),
                    category_description: "".to_string(),
                },
                proposal_title: "the proposal".to_string(),
                proposal_summary: "the proposal summary".to_string(),
                proposal_public_key: "pubkey".to_string(),
                proposal_funds: 10000,
                proposal_url: "http://foo.bar".to_string(),
                proposal_files_url: "http://foo.bar/files".to_string(),
                proposal_impact_score: 100,
                reviews_count: 0,
                proposer: Proposer {
                    proposer_name: "tester".to_string(),
                    proposer_email: "tester@tester.tester".to_string(),
                    proposer_url: "http://tester.tester".to_string(),
                    proposer_relevant_experience: "ilumination".to_string(),
                },
                chain_proposal_id: b"foobar".to_vec(),
                chain_vote_options: VoteOptions::parse_coma_separated_value("b,a,r"),
                chain_vote_start_time: OffsetDateTime::now_utc().unix_timestamp(),
                chain_vote_end_time: OffsetDateTime::now_utc().unix_timestamp(),
                chain_committee_end_time: OffsetDateTime::now_utc().unix_timestamp(),
                chain_voteplan_payload: "none".to_string(),
                chain_vote_encryption_key: "none".to_string(),
                fund_id: 1,
                challenge_id: CHALLENGE_ID,
                extra: Some(
                    vec![("key1", "value1"), ("key2", "value2")]
                        .into_iter()
                        .map(|(a, b)| (a.to_string(), b.to_string()))
                        .collect(),
                ),
            },
            challenge_info: ProposalChallengeInfo::CommunityChoice(
                community_choice::ChallengeInfo {
                    proposal_brief: "A for ADA".to_string(),
                    proposal_importance: "We need to get them while they're young.".to_string(),
                    proposal_goal: "Nebulous".to_string(),
                    proposal_metrics:
                        "\\- Number of people engaged into the creation of Cryptoalphabet"
                            .to_string(),
                },
            ),
            challenge_type: ChallengeType::CommunityChoice,
            voteplan: ProposalVotePlanCommon {
                chain_proposal_index: 0,
                chain_voteplan_id: "voteplain_id".to_string(),
            },
            group_id: group_id.into(),
        }
    }

    pub fn populate_db_with_proposal(full_proposal: &FullProposalInfo, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();
        populate_db_with_proposal_conn(full_proposal, &connection);
    }

    pub fn populate_db_with_proposal_conn(
        full_proposal: &FullProposalInfo,
        connection: &DbConnection,
    ) {
        let proposal = &full_proposal.proposal;
        let proposal_id = proposal.proposal_id.clone();
        // insert the proposal information
        let values = (
            proposals::proposal_id.eq(proposal.proposal_id.clone()),
            proposals::proposal_category.eq(proposal.proposal_category.category_name.clone()),
            proposals::proposal_title.eq(proposal.proposal_title.clone()),
            proposals::proposal_summary.eq(proposal.proposal_summary.clone()),
            proposals::proposal_public_key.eq(proposal.proposal_public_key.clone()),
            proposals::proposal_funds.eq(proposal.proposal_funds),
            proposals::proposal_url.eq(proposal.proposal_url.clone()),
            proposals::proposal_files_url.eq(proposal.proposal_files_url.clone()),
            proposals::proposal_impact_score.eq(proposal.proposal_impact_score),
            proposals::proposer_name.eq(proposal.proposer.proposer_name.clone()),
            proposals::proposer_contact.eq(proposal.proposer.proposer_email.clone()),
            proposals::proposer_url.eq(proposal.proposer.proposer_url.clone()),
            proposals::proposer_relevant_experience
                .eq(proposal.proposer.proposer_relevant_experience.clone()),
            proposals::chain_proposal_id.eq(proposal.chain_proposal_id.clone()),
            proposals::chain_vote_options.eq(proposal.chain_vote_options.as_csv_string()),
            proposals::challenge_id.eq(proposal.challenge_id),
            proposals::extra.eq(proposal
                .extra
                .as_ref()
                .map(|h| serde_json::to_string(h).unwrap())),
        );

        diesel::insert_into(proposals::table)
            .values(values)
            .execute(connection)
            .unwrap();

        let token_identifier = format!("{}-token", full_proposal.group_id);

        // insert the related fund voteplan information
        let voteplan_values = (
            voteplans::chain_voteplan_id.eq(full_proposal.voteplan.chain_voteplan_id.clone()),
            voteplans::chain_vote_start_time.eq(proposal.chain_vote_start_time),
            voteplans::chain_vote_end_time.eq(proposal.chain_vote_end_time),
            voteplans::chain_committee_end_time.eq(proposal.chain_committee_end_time),
            voteplans::chain_voteplan_payload.eq(proposal.chain_voteplan_payload.clone()),
            voteplans::chain_vote_encryption_key.eq(proposal.chain_vote_encryption_key.clone()),
            voteplans::fund_id.eq(proposal.fund_id),
            voteplans::token_identifier.eq(&token_identifier),
        );

        diesel::insert_into(voteplans::table)
            .values(voteplan_values)
            .execute(connection)
            .unwrap();

        diesel::insert_into(crate::db::schema::groups::table)
            .values(
                Group {
                    fund_id: proposal.fund_id,
                    group_id: full_proposal.group_id.clone(),
                    token_identifier,
                }
                .values(),
            )
            .execute(connection)
            .unwrap();

        let proposal_voteplan_values = (
            proposals_voteplans::proposal_id.eq(proposal_id),
            proposals_voteplans::chain_voteplan_id
                .eq(full_proposal.voteplan.chain_voteplan_id.clone()),
            proposals_voteplans::chain_proposal_index
                .eq(full_proposal.voteplan.chain_proposal_index),
        );

        diesel::insert_into(proposals_voteplans::table)
            .values(proposal_voteplan_values)
            .execute(connection)
            .unwrap();

        match &full_proposal.challenge_info {
            ProposalChallengeInfo::Simple(data) => {
                let simple_values = (
                    proposal_simple_challenge::proposal_id.eq(proposal.proposal_id.clone()),
                    proposal_simple_challenge::proposal_solution.eq(data.proposal_solution.clone()),
                );

                diesel::insert_into(proposal_simple_challenge::table)
                    .values(simple_values)
                    .execute(connection)
                    .unwrap();
            }
            ProposalChallengeInfo::CommunityChoice(data) => {
                let community_values = (
                    proposal_community_choice_challenge::proposal_id
                        .eq(proposal.proposal_id.clone()),
                    proposal_community_choice_challenge::proposal_brief
                        .eq(data.proposal_brief.clone()),
                    proposal_community_choice_challenge::proposal_importance
                        .eq(data.proposal_importance.clone()),
                    proposal_community_choice_challenge::proposal_goal
                        .eq(data.proposal_goal.clone()),
                    proposal_community_choice_challenge::proposal_metrics
                        .eq(data.proposal_metrics.clone()),
                );

                diesel::insert_into(proposal_community_choice_challenge::table)
                    .values(community_values)
                    .execute(connection)
                    .unwrap();
            }
        };
    }
}
