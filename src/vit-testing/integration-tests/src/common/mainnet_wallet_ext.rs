use crate::common::CardanoWallet;
use snapshot_lib::VoterHIR;
use vitup::config::Block0Initial;

pub trait MainnetWalletExtension {
    fn as_initial_entry(&self) -> Block0Initial;
    fn as_voter_hir(&self, group: &str) -> VoterHIR;
}

impl MainnetWalletExtension for CardanoWallet {
    fn as_initial_entry(&self) -> Block0Initial {
        Block0Initial::External {
            address: self.catalyst_address().to_string(),
            funds: self.stake(),
            role: Default::default(),
        }
    }

    fn as_voter_hir(&self, group: &str) -> VoterHIR {
        VoterHIR {
            voting_key: self.catalyst_public_key(),
            voting_group: group.to_string(),
            voting_power: self.stake().into(),
            address: self.catalyst_address().into(),
            underthreshold: false,
            overlimit: false,
            private_key: None,
        }
    }
}
