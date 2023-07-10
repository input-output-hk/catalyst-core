use super::SerdeType;
use event_db::types::event::ballot::{Ballot, ObjectiveBallots, ProposalBallot, VotePlan};
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};

impl Serialize for SerdeType<&VotePlan> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("VotePlan", 5)?;
        serializer.serialize_field("chain_proposal_index", &self.chain_proposal_index)?;
        serializer.serialize_field("group", &SerdeType(&self.group))?;
        serializer.serialize_field("ballot_type", &self.ballot_type.0)?;
        serializer.serialize_field("chain_voteplan_id", &self.chain_voteplan_id)?;
        if let Some(encryption_key) = &self.encryption_key {
            serializer.serialize_field("encryption_key", encryption_key)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<VotePlan> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Ballot> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("Ballot", 2)?;
        serializer.serialize_field("choices", &self.choices.0)?;
        serializer.serialize_field(
            "voteplans",
            &self.voteplans.0.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        serializer.end()
    }
}

impl Serialize for SerdeType<Ballot> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ProposalBallot> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ProposalBallot", 2)?;
        serializer.serialize_field("proposal_id", &SerdeType(&self.proposal_id))?;
        serializer.serialize_field("ballot", &SerdeType(&self.ballot))?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ProposalBallot> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ObjectiveBallots> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ObjectiveBallots", 2)?;
        serializer.serialize_field("objective_id", &SerdeType(&self.objective_id))?;
        serializer.serialize_field(
            "ballots",
            &self.ballots.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ObjectiveBallots> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_db::types::{
        event::{
            ballot::{BallotType, GroupVotePlans, ObjectiveChoices},
            objective::ObjectiveId,
            proposal::ProposalId,
        },
        registration::VoterGroupId,
    };
    use serde_json::json;

    #[test]
    fn vote_plan_json_test() {
        let vote_plan = SerdeType(VotePlan {
            chain_proposal_index: 1,
            group: VoterGroupId("rep".to_string()),
            ballot_type: BallotType("public".to_string()),
            chain_voteplan_id: "chain_voteplan_id 1".to_string(),
            encryption_key: Some("encryption_key 1".to_string()),
        });

        let json = serde_json::to_value(&vote_plan).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "chain_proposal_index": 1,
                    "group": "rep",
                    "ballot_type": "public",
                    "chain_voteplan_id": "chain_voteplan_id 1",
                    "encryption_key": "encryption_key 1"
                }
            )
        );

        let vote_plan = SerdeType(VotePlan {
            chain_proposal_index: 1,
            group: VoterGroupId("rep".to_string()),
            ballot_type: BallotType("public".to_string()),
            chain_voteplan_id: "chain_voteplan_id 1".to_string(),
            encryption_key: None,
        });

        let json = serde_json::to_value(&vote_plan).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "chain_proposal_index": 1,
                    "group": "rep",
                    "ballot_type": "public",
                    "chain_voteplan_id": "chain_voteplan_id 1",
                }
            )
        );
    }

    #[test]
    fn ballot_json_test() {
        let ballot = SerdeType(Ballot {
            choices: ObjectiveChoices(vec![]),
            voteplans: GroupVotePlans(vec![]),
        });

        let json = serde_json::to_value(&ballot).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "choices": [],
                    "voteplans": []
                }
            )
        );
    }

    #[test]
    fn proposal_ballot_json_test() {
        let ballot = SerdeType(ProposalBallot {
            proposal_id: ProposalId(1),
            ballot: Ballot {
                choices: ObjectiveChoices(vec![]),
                voteplans: GroupVotePlans(vec![]),
            },
        });

        let json = serde_json::to_value(&ballot).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "proposal_id": 1,
                    "ballot": {
                        "choices": [],
                        "voteplans": []
                    }
                }
            )
        );
    }

    #[test]
    fn objective_ballots_json_test() {
        let ballot = SerdeType(ObjectiveBallots {
            objective_id: ObjectiveId(1),
            ballots: vec![],
        });

        let json = serde_json::to_value(&ballot).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "objective_id": 1,
                    "ballots": []
                }
            )
        );
    }
}
