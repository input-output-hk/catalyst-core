use poem_openapi::{NewType, Object};
use serde::Deserialize;

/// A Voting Key.
#[derive(NewType, Deserialize)]
pub struct VotingKey(pub String);

/// Voter Group ID.
/// `direct` = Direct voter.
/// `rep` = Delegated Representative.
#[derive(NewType)]
pub struct VoterGroupId(pub String);

impl From<event_db::types::registration::VoterGroupId> for VoterGroupId {
    fn from(value: event_db::types::registration::VoterGroupId) -> Self {
        Self(value.0)
    }
}

/// Voter Info
#[derive(Object)]
pub struct VoterInfo {
    /// Voter's voting power.
    /// This is the true voting power, subject to minimum voting power and max cap.
    voting_power: i64,
    voting_group: VoterGroupId,
    /// Total voting power delegated to this voter.
    /// This is not capped and not subject to minimum voting power.
    delegations_power: i64,
    /// Number of registration which delegated to this voter.
    delegations_count: i64,
    /// Voting power's share of the total voting power.
    /// Can be used to gauge potential voting power saturation.
    /// This value is NOT saturated however, and gives the raw share of total registered voting power.
    voting_power_saturation: f64,
    #[oai(skip_serializing_if = "Option::is_none")]
    /// List of stake public key addresses which delegated to this voting key.
    delegator_addresses: Option<Vec<String>>,
}

impl From<event_db::types::registration::VoterInfo> for VoterInfo {
    fn from(value: event_db::types::registration::VoterInfo) -> Self {
        Self {
            voting_power: value.voting_power,
            voting_group: value.voting_group.into(),
            delegations_power: value.delegations_power,
            delegations_count: value.delegations_count,
            voting_power_saturation: value.voting_power_saturation,
            delegator_addresses: value.delegator_addresses,
        }
    }
}

/// Voter
#[derive(Object)]
pub struct Voter {
    voter_info: VoterInfo,
    // #[oai(serialize_with = "serialize_datetime_as_rfc3339")]
    // as_at: DateTime<Utc>,
    // #[oai(serialize_with = "serialize_datetime_as_rfc3339")]
    // last_updated: DateTime<Utc>,
    #[oai(rename = "final")]
    is_final: bool,
}

impl From<event_db::types::registration::Voter> for Voter {
    fn from(value: event_db::types::registration::Voter) -> Self {
        Self {
            voter_info: value.voter_info.into(),
            is_final: value.is_final,
        }
    }
}
