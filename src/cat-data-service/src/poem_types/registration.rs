use chrono::{DateTime, Utc};
use poem_openapi::{NewType, Object};
use serde::Deserialize;

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
    #[oai(skip_serializing_if_is_none = true)]
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
    /// Date and time the latest snapshot represents.
    as_at: DateTime<Utc>,
    /// Date and time for the latest update to this snapshot information.
    last_updated: DateTime<Utc>,
    /// `True` - this is the final snapshot which will be used for voting power in the event.
    /// `False` - this is an interim snapshot, subject to change.
    #[oai(rename = "final")]
    is_final: bool,
}

impl From<event_db::types::registration::Voter> for Voter {
    fn from(value: event_db::types::registration::Voter) -> Self {
        Self {
            voter_info: value.voter_info.into(),
            as_at: value.as_at,
            last_updated: value.last_updated,
            is_final: value.is_final,
        }
    }
}

/// Voter's delegation info
#[derive(Object)]
pub struct Delegation {
    /// Hex encoded voting key for this delegation.
    voting_key: String,
    group: VoterGroupId,
    /// Relative weight assigned to this voting key.
    weight: i32,
    /// Raw voting power distributed to this voting key.
    value: i64,
}

impl From<event_db::types::registration::Delegation> for Delegation {
    fn from(value: event_db::types::registration::Delegation) -> Self {
        Self {
            voting_key: value.voting_key,
            group: value.group.into(),
            weight: value.weight,
            value: value.value,
        }
    }
}

#[derive(Object)]
pub struct RewardAddress {
    /// Reward address for this delegation.
    reward_address: String,
    /// Flag which reflects does the `reward_address` valid or not, contains it "addr" or "addr_test" prefix or not.
    reward_payable: bool,
}

impl From<event_db::types::registration::RewardAddress> for RewardAddress {
    fn from(value: event_db::types::registration::RewardAddress) -> Self {
        Self {
            reward_address: value.reward_address(),
            reward_payable: value.reward_payable(),
        }
    }
}

#[derive(Object)]
pub struct Delegator {
    /// List off delegations made by this stake address.
    /// In the order as presented in the voters registration.
    delegations: Vec<Delegation>,
    #[oai(flatten)]
    reward_address: RewardAddress,
    /// Raw total voting power from stake address.
    raw_power: i64,
    /// Total voting power, across all registered voters.
    total_power: i64,
    /// Date and time for the latest update to this snapshot information.
    as_at: DateTime<Utc>,
    /// Date and time the latest snapshot represents.
    last_updated: DateTime<Utc>,
    /// `True` - this is the final snapshot which will be used for voting power in the event.
    /// `False`- this is an interim snapshot, subject to change.
    #[oai(rename = "final")]
    is_final: bool,
}

impl From<event_db::types::registration::Delegator> for Delegator {
    fn from(value: event_db::types::registration::Delegator) -> Self {
        Self {
            delegations: value.delegations.into_iter().map(Into::into).collect(),
            reward_address: value.reward_address.into(),
            raw_power: value.raw_power,
            total_power: value.total_power,
            as_at: value.as_at,
            last_updated: value.last_updated,
            is_final: value.is_final,
        }
    }
}
