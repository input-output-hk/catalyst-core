use super::registration::VoterGroupId;
use poem_openapi::{NewType, Object};
use serde::Deserialize;

/// The kind of ballot to be cast on this Objective.
/// * `public` - All Ballots are public when cast.
/// * `private` - All Ballots are private.
/// * `cast-private` - All Ballots are cast privately but become public after the tally.
#[derive(NewType, Deserialize)]
pub struct BallotType(String);

impl From<event_db::types::ballot::BallotType> for BallotType {
    fn from(value: event_db::types::ballot::BallotType) -> Self {
        Self(value.0)
    }
}

/// The voteplan to use for this group.
#[derive(Object)]
pub struct VotePlan {
    /// The Index of the proposal, needed to create a ballot for it.
    chain_proposal_index: i64,

    /// The name of the group.
    #[oai(skip_serializing_if_is_none = true)]
    group: Option<VoterGroupId>,

    /// The type of ballot this group must cast.
    ballot_type: BallotType,

    /// Blockchain ID of the vote plan transaction.
    chain_voteplan_id: String,

    /// The public encryption key used. ONLY if required by the ballot type (private, cast-private).
    #[oai(skip_serializing_if_is_none = true)]
    encryption_key: Option<String>,
}

impl From<event_db::types::ballot::VotePlan> for VotePlan {
    fn from(value: event_db::types::ballot::VotePlan) -> Self {
        Self {
            chain_proposal_index: value.chain_proposal_index,
            group: value.group.map(Into::into),
            ballot_type: value.ballot_type.into(),
            chain_voteplan_id: value.chain_voteplan_id,
            encryption_key: value.encryption_key,
        }
    }
}

/// Details necessary to complete a ballot for the specific proposal and objective.
#[derive(Object)]
pub struct Ballot {
    /// Ballot Choices present for all proposals in this Objective.
    ///
    /// Ordered list of choices available for all proposals in this Objective.
    /// The offset into the array is the index of the choice.
    /// For example, the first element is Choice 0, second is Choice 1 and so on.
    choices: Vec<String>,

    /// List of groups and the voteplans they use when voting on this proposal.
    /// Each valid group for this Objective:
    /// * Must be listed.
    /// * Must not be repeated.
    voteplans: Vec<VotePlan>,
}
