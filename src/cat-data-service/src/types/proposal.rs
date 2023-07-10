use super::SerdeType;
use event_db::types::proposal::{
    Proposal, ProposalDetails, ProposalId, ProposalSummary, ProposerDetails,
};
use serde::{
    de::Deserializer,
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

impl Serialize for SerdeType<&ProposalId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0 .0.serialize(serializer)
    }
}

impl Serialize for SerdeType<ProposalId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerdeType<ProposalId> {
    fn deserialize<D>(deserializer: D) -> Result<SerdeType<ProposalId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ProposalId(i32::deserialize(deserializer)?).into())
    }
}

impl Serialize for SerdeType<&ProposalSummary> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ProposalSummary", 3)?;
        serializer.serialize_field("id", &SerdeType(&self.id))?;
        serializer.serialize_field("title", &self.title)?;
        serializer.serialize_field("summary", &self.summary)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ProposalSummary> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ProposerDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ProposerDetails", 4)?;
        serializer.serialize_field("name", &self.name)?;
        serializer.serialize_field("email", &self.email)?;
        serializer.serialize_field("url", &self.url)?;
        serializer.serialize_field("payment_key", &self.payment_key)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ProposerDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ProposalDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ProposalDetails", 5)?;
        serializer.serialize_field("funds", &self.funds)?;
        serializer.serialize_field("url", &self.url)?;
        serializer.serialize_field("files", &self.files)?;
        serializer.serialize_field(
            "proposer",
            &self.proposer.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        if let Some(supplemental) = &self.supplemental {
            serializer.serialize_field("supplemental", supplemental)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<ProposalDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Proposal> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        pub struct ProposalImpl<'a> {
            #[serde(flatten)]
            summary: SerdeType<&'a ProposalSummary>,
            #[serde(flatten)]
            details: SerdeType<&'a ProposalDetails>,
        }

        let val = ProposalImpl {
            summary: SerdeType(&self.summary),
            details: SerdeType(&self.details),
        };
        val.serialize(serializer)
    }
}

impl Serialize for SerdeType<Proposal> {
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
    use serde_json::json;

    #[test]
    fn proposal_id_json_test() {
        let proposal_id = SerdeType(ProposalId(1));

        let json = serde_json::to_value(&proposal_id).unwrap();
        assert_eq!(json, json!(1));

        let expected: SerdeType<ProposalId> = serde_json::from_value(json).unwrap();
        assert_eq!(expected, proposal_id);
    }

    #[test]
    fn proposal_summary_json_test() {
        let proposal_summary = SerdeType(ProposalSummary {
            id: ProposalId(1),
            title: "title".to_string(),
            summary: "summary".to_string(),
        });

        let json = serde_json::to_value(&proposal_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "title": "title",
                    "summary": "summary",
                }
            )
        )
    }

    #[test]
    fn proposer_details_json_test() {
        let proposer_details = SerdeType(ProposerDetails {
            name: "proposer name".to_string(),
            email: "proposer email".to_string(),
            url: "proposer url".to_string(),
            payment_key: "proposer payment key".to_string(),
        });

        let json = serde_json::to_value(&proposer_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "name": "proposer name",
                    "email": "proposer email",
                    "url": "proposer url",
                    "payment_key": "proposer payment key",
                }
            )
        );
    }

    #[test]
    fn proposal_details_json_test() {
        let proposal_details = SerdeType(ProposalDetails {
            funds: 1,
            url: "url".to_string(),
            files: "files".to_string(),
            proposer: vec![],
            supplemental: Some(json!({})),
        });

        let json = serde_json::to_value(&proposal_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "funds": 1,
                    "url": "url",
                    "files": "files",
                    "proposer": [],
                    "supplemental": {}
                }
            )
        );

        let proposal_details = SerdeType(ProposalDetails {
            funds: 1,
            url: "url".to_string(),
            files: "files".to_string(),
            proposer: vec![],
            supplemental: None,
        });

        let json = serde_json::to_value(&proposal_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "funds": 1,
                    "url": "url",
                    "files": "files",
                    "proposer": [],
                }
            )
        )
    }

    #[test]
    fn proposal_json_test() {
        let proposal = SerdeType(Proposal {
            summary: ProposalSummary {
                id: ProposalId(1),
                title: "title".to_string(),
                summary: "summary".to_string(),
            },
            details: ProposalDetails {
                funds: 1,
                url: "url".to_string(),
                files: "files".to_string(),
                proposer: vec![],
                supplemental: None,
            },
        });

        let json = serde_json::to_value(&proposal).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "title": "title",
                    "summary": "summary",
                    "funds": 1,
                    "url": "url",
                    "files": "files",
                    "proposer": [],
                }
            )
        )
    }
}
