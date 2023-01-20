use crate::{
    data::{NetworkId, VotingPurpose},
    SlotNo,
};

/// Arguments to the `voting_power` function
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VotingPowerArgs {
    /// The lower bound for slots to include
    pub min_slot: Option<SlotNo>,
    /// The upper bound for slots to include
    pub max_slot: Option<SlotNo>,
    /// The network to validate addresses against
    pub network_id: NetworkId,
    /// The voting purpose we expect registrations to have
    pub expected_voting_purpose: VotingPurpose,
}

impl Default for VotingPowerArgs {
    fn default() -> Self {
        Self {
            min_slot: None,
            max_slot: None,
            network_id: NetworkId::Mainnet,
            expected_voting_purpose: VotingPurpose::CATALYST,
        }
    }
}
