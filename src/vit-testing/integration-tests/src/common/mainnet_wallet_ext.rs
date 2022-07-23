use crate::common::MainnetWallet;
use vitup::config::Block0Initial;
use voting_hir::VoterHIR;

pub trait MainnetWalletExtension {
    fn as_initial_entry(&self) -> Block0Initial;
    fn as_voter_hir(&self, group: &str) -> VoterHIR;
}

impl MainnetWalletExtension for MainnetWallet {
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
        }
    }
}
