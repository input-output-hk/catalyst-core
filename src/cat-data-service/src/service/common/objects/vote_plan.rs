//! Defines the vote plan.
//!
use super::{ballot_type::BallotType, voter_group_id::VoterGroupId};
use poem_openapi::{types::Example, Object};

/// The vote plan assigned to a voter group.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct VotePlan {
    /// The Index of the proposal, needed to create a ballot for it.
    #[oai(validator(minimum(value = "0")))]
    chain_proposal_index: i64,

    /// The name of the group.
    #[oai(skip_serializing_if_is_none = true)]
    group: Option<VoterGroupId>,

    /// The type of ballot this group must cast.
    ballot_type: BallotType,

    /// Blockchain ID of the vote plan transaction.
    #[oai(validator(max_length = 66, min_length = 66, pattern = "0x[0-9a-f]{64}"))]
    chain_voteplan_id: String,

    /// The public encryption key used. ONLY if required by the ballot type (private, cast-private).
    #[oai(
        skip_serializing_if_is_none = true,
        validator(max_length = 66, min_length = 66, pattern = "0x[0-9a-f]{64}")
    )]
    encryption_key: Option<String>,
}

impl Example for VotePlan {
    fn example() -> Self {
        Self {
            chain_proposal_index: 0,
            group: None,
            ballot_type: BallotType::example(),
            chain_voteplan_id: "0xad6eaebafd2cca7e1829df26c57b340a98b9d513b7eddec8561883f1b99f3b9e"
                .to_string(),
            encryption_key: Some(
                "0xbc5fdebafd2cca7e1829df26c57b340a98b9d513b7eddec8561883f1b99f3b9e".to_string(),
            ),
        }
    }
}

impl TryFrom<event_db::types::ballot::VotePlan> for VotePlan {
    type Error = String;
    fn try_from(value: event_db::types::ballot::VotePlan) -> Result<Self, Self::Error> {
        let group = if let Some(group) = value.group {
            Some(group.try_into()?)
        } else {
            None
        };
        Ok(Self {
            chain_proposal_index: value.chain_proposal_index,
            group,
            ballot_type: value.ballot_type.try_into()?,
            chain_voteplan_id: value.chain_voteplan_id,
            encryption_key: value.encryption_key,
        })
    }
}
