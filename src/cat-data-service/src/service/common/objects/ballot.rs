//! Defines the Ballot type.
//!
use super::vote_plan::VotePlan;
use poem_openapi::{types::Example, Object};

/// Details necessary to complete a ballot for the specific proposal and objective.
#[derive(Object)]
#[oai(example = true)]
pub struct Ballot {
    /// Ballot Choices present for all proposals in this Objective.
    ///
    /// Ordered list of choices available for all proposals in this Objective.
    /// The offset into the array is the index of the choice.
    choices: Vec<String>,

    /// List of groups and the voteplans they use when voting on this proposal.
    /// Each valid group for this Objective:
    /// * Must be listed.
    /// * Must not be repeated.
    voteplans: Vec<VotePlan>,
}

impl Example for Ballot {
    fn example() -> Self {
        Self {
            choices: vec!["yes".to_string(), "no".to_string(), "abstain".to_string()],
            voteplans: vec![VotePlan::example()],
        }
    }
}

impl TryFrom<event_db::types::ballot::Ballot> for Ballot {
    type Error = String;
    fn try_from(value: event_db::types::ballot::Ballot) -> Result<Self, Self::Error> {
        let mut voteplans = Vec::new();
        for voteplan in value.voteplans.0 {
            voteplans.push(voteplan.try_into()?);
        }
        Ok(Self {
            choices: value.choices.0,
            voteplans,
        })
    }
}
